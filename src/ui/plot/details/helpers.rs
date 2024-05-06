use std::ptr;

pub fn make_string(data: Vec<i8>) -> String {
    data.into_iter()
        .map(|c| c as u8 as char)
        .collect::<String>()
}

pub fn get_shader_info_log(shader_id: u32) -> String {
    let mut info_log_length = 0;
    unsafe {
        gl::GetShaderiv(
            shader_id,
            gl::INFO_LOG_LENGTH,
            ptr::addr_of_mut!(info_log_length),
        )
    };

    let mut info_log = vec![0; info_log_length as usize];

    unsafe {
        gl::GetShaderInfoLog(
            shader_id,
            info_log_length,
            0 as *mut i32,
            info_log.as_mut_ptr(),
        )
    };

    make_string(info_log)
}

pub fn check_shader_compilation_status(shader_id: u32) -> Result<(), String> {
    let mut compilation_status = 0;
    unsafe {
        gl::GetShaderiv(
            shader_id,
            gl::COMPILE_STATUS,
            ptr::addr_of_mut!(compilation_status),
        )
    };

    if compilation_status == gl::FALSE as i32 {
        return Err(get_shader_info_log(shader_id));
    }

    Ok(())
}

pub fn check_shader_linking_status(program_id: u32) -> Result<(), String> {
    let mut link_status = 0;
    unsafe { gl::GetProgramiv(program_id, gl::LINK_STATUS, ptr::addr_of_mut!(link_status)) };

    if link_status == gl::FALSE as i32 {
        return Err(get_shader_info_log(program_id));
    }

    Ok(())
}
