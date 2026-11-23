use super::*;

pub unsafe fn default_audio_endpoint() -> Option<IAudioEndpointVolume> {
    let class_id = GUID::from_u128(0xbcde0395_e52f_467c_8e3d_c4579291692e);
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&class_id, None, CLSCTX_ALL).ok()?;
    let device = enumerator
        .GetDefaultAudioEndpoint(eRender, eMultimedia)
        .ok()?;
    device.Activate(CLSCTX_ALL, None).ok()
}

pub unsafe fn active_audio_session_count() -> u32 {
    let class_id = GUID::from_u128(0xbcde0395_e52f_467c_8e3d_c4579291692e);
    let enumerator: IMMDeviceEnumerator = match CoCreateInstance(&class_id, None, CLSCTX_ALL) {
        Ok(value) => value,
        Err(_) => return 0,
    };
    let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) else {
        return 0;
    };
    let Ok(manager) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) else {
        return 0;
    };
    let Ok(sessions) = manager.GetSessionEnumerator() else {
        return 0;
    };
    let Ok(count) = sessions.GetCount() else {
        return 0;
    };
    (0..count)
        .filter_map(|index| sessions.GetSession(index).ok())
        .filter_map(|session| session.GetState().ok())
        .filter(|state| *state == AudioSessionStateActive)
        .count() as u32
}

pub struct AudioSessionRow {
    pub label: String,
    pub volume: u8,
    pub muted: bool,
}

pub unsafe fn process_label(pid: u32) -> String {
    if pid == 0 {
        return "System audio".to_string();
    }
    let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
        return format!("Process {}", pid);
    };
    let mut buffer = vec![0u16; 512];
    let mut length = buffer.len() as u32;
    let label = if QueryFullProcessImageNameW(
        process,
        PROCESS_NAME_FORMAT(0),
        PWSTR(buffer.as_mut_ptr()),
        &mut length,
    )
    .is_ok()
    {
        PathBuf::from(String::from_utf16_lossy(&buffer[..length as usize]))
            .file_stem()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    } else {
        None
    };
    let _ = CloseHandle(process);
    label.unwrap_or_else(|| format!("Process {}", pid))
}

pub unsafe fn audio_session_controls() -> Vec<(IAudioSessionControl2, ISimpleAudioVolume)> {
    let class_id = GUID::from_u128(0xbcde0395_e52f_467c_8e3d_c4579291692e);
    let Ok(enumerator): Result<IMMDeviceEnumerator, _> =
        CoCreateInstance(&class_id, None, CLSCTX_ALL)
    else {
        return Vec::new();
    };
    let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) else {
        return Vec::new();
    };
    let Ok(manager) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) else {
        return Vec::new();
    };
    let Ok(sessions) = manager.GetSessionEnumerator() else {
        return Vec::new();
    };
    let Ok(count) = sessions.GetCount() else {
        return Vec::new();
    };
    (0..count)
        .filter_map(|index| sessions.GetSession(index).ok())
        .filter_map(|session| {
            if session.GetState().ok()? != AudioSessionStateActive {
                return None;
            }
            let control = session.cast::<IAudioSessionControl2>().ok()?;
            let volume = session.cast::<ISimpleAudioVolume>().ok()?;
            Some((control, volume))
        })
        .collect()
}

pub unsafe fn audio_session_rows() -> Vec<AudioSessionRow> {
    audio_session_controls()
        .into_iter()
        .map(|(control, volume)| {
            let pid = control.GetProcessId().unwrap_or(0);
            let level = (volume.GetMasterVolume().unwrap_or(0.0) * 100.0)
                .round()
                .clamp(0.0, 100.0) as u8;
            let muted = volume
                .GetMute()
                .map(|value| value.as_bool())
                .unwrap_or(false);
            AudioSessionRow {
                label: process_label(pid),
                volume: level,
                muted,
            }
        })
        .collect()
}

pub unsafe fn toggle_audio_session(index: usize) -> bool {
    let Some((_, volume)) = audio_session_controls().into_iter().nth(index) else {
        return false;
    };
    let Ok(muted) = volume.GetMute() else {
        return false;
    };
    let context = GUID::from_u128(0);
    volume.SetMute(!muted.as_bool(), &context).is_ok()
}

pub unsafe fn adjust_audio_session(index: usize, delta: f32) -> bool {
    let Some((_, volume)) = audio_session_controls().into_iter().nth(index) else {
        return false;
    };
    let Ok(current) = volume.GetMasterVolume() else {
        return false;
    };
    let context = GUID::from_u128(0);
    volume
        .SetMasterVolume((current + delta).clamp(0.0, 1.0), &context)
        .is_ok()
}

pub unsafe fn default_audio_state() -> Option<(u8, bool)> {
    let endpoint = default_audio_endpoint()?;
    let level = (endpoint.GetMasterVolumeLevelScalar().ok()? * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8;
    let muted = endpoint.GetMute().ok()?.as_bool();
    Some((level, muted))
}

pub unsafe fn toggle_audio_mute() -> bool {
    let Some(endpoint) = default_audio_endpoint() else {
        return false;
    };
    let Ok(muted) = endpoint.GetMute() else {
        return false;
    };
    let context = GUID::from_u128(0);
    endpoint.SetMute(!muted.as_bool(), &context).is_ok()
}

pub unsafe fn adjust_master_volume(delta: f32) -> bool {
    let Some(endpoint) = default_audio_endpoint() else {
        return false;
    };
    let Ok(current) = endpoint.GetMasterVolumeLevelScalar() else {
        return false;
    };
    let target = (current + delta).clamp(0.0, 1.0);
    let context = GUID::from_u128(0);
    endpoint
        .SetMasterVolumeLevelScalar(target, &context)
        .is_ok()
}

#[repr(C)]
pub struct AudioVolumeNotificationData {
    guid_event_context: GUID,
    muted: BOOL,
    master_volume: f32,
    channels: u32,
    channel_volumes: [f32; 1],
}

#[repr(C)]
pub struct CallbackVtbl {
    query_interface: unsafe extern "system" fn(
        *mut std::ffi::c_void,
        *const GUID,
        *mut *mut std::ffi::c_void,
    ) -> HRESULT,
    add_ref: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    on_notify: unsafe extern "system" fn(
        *mut std::ffi::c_void,
        *const AudioVolumeNotificationData,
    ) -> HRESULT,
}

unsafe impl Sync for CallbackVtbl {}
unsafe impl Send for CallbackVtbl {}

#[repr(C)]
pub struct VolumeCallbackImpl {
    vtbl: *const CallbackVtbl,
}

unsafe impl Sync for VolumeCallbackImpl {}
unsafe impl Send for VolumeCallbackImpl {}

unsafe extern "system" fn volume_query_interface(
    this: *mut std::ffi::c_void,
    riid: *const GUID,
    ppv: *mut *mut std::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return windows::Win32::Foundation::E_POINTER;
    }
    let iid_unk = GUID::from_u128(0x00000000_0000_0000_c000_000000000046);
    let iid_cb = GUID::from_u128(0x657804fa_d6ad_4496_8a60_352752af4f89);
    unsafe {
        if *riid == iid_unk || *riid == iid_cb {
            *ppv = this;
            return windows::Win32::Foundation::S_OK;
        }
        *ppv = std::ptr::null_mut();
    }
    windows::Win32::Foundation::E_NOINTERFACE
}

unsafe extern "system" fn volume_add_ref(_this: *mut std::ffi::c_void) -> u32 {
    1
}

unsafe extern "system" fn volume_release(_this: *mut std::ffi::c_void) -> u32 {
    1
}

unsafe extern "system" fn volume_on_notify(
    _this: *mut std::ffi::c_void,
    _p_notify: *const AudioVolumeNotificationData,
) -> HRESULT {
    unsafe {
        if !PANEL.is_invalid() {
            let _ = PostMessageW(PANEL, WM_APP_AUDIO_CHANGE, WPARAM(0), LPARAM(0));
        }
    }
    windows::Win32::Foundation::S_OK
}

pub static VOLUME_CALLBACK_VTBL: CallbackVtbl = CallbackVtbl {
    query_interface: volume_query_interface,
    add_ref: volume_add_ref,
    release: volume_release,
    on_notify: volume_on_notify,
};

pub static VOLUME_CALLBACK_INSTANCE: VolumeCallbackImpl = VolumeCallbackImpl {
    vtbl: &VOLUME_CALLBACK_VTBL,
};

pub unsafe fn register_volume_callback() {
    if let Some(endpoint) = default_audio_endpoint() {
        let cb_ptr =
            &VOLUME_CALLBACK_INSTANCE as *const VolumeCallbackImpl as *mut std::ffi::c_void;
        let cb: &IAudioEndpointVolumeCallback = std::mem::transmute(&cb_ptr);
        if endpoint.RegisterControlChangeNotify(cb).is_ok() {
            REGISTERED_ENDPOINT = Some(endpoint);
            log_event("registered audio endpoint volume callback");
        }
    }
}

pub unsafe fn unregister_volume_callback() {
    if let Some(endpoint) = REGISTERED_ENDPOINT.take() {
        let cb_ptr =
            &VOLUME_CALLBACK_INSTANCE as *const VolumeCallbackImpl as *mut std::ffi::c_void;
        let cb: &IAudioEndpointVolumeCallback = std::mem::transmute(&cb_ptr);
        let _ = endpoint.UnregisterControlChangeNotify(cb);
        log_event("unregistered audio endpoint volume callback");
    }
}
