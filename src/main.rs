use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::sync::mpsc::Receiver;
use glfw::{Action, Context, Key};
use gl::types::*;
use std::ptr;


fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        if let glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) = event {
                window.set_should_close(true)
        }
    }
}


fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let (mut window, events) = glfw
        .create_window(800, 800, "opengl", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");
    window.set_key_polling(true);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s) as *const _);
    gl::Viewport::load_with(|s| window.get_proc_address(s) as *const _);

    let (shader_program, vao) = unsafe {
        const VERTEX_SHADER_SOURCE: &str = r#"
            #version 330 core
            layout (location = 0) in vec3 position;
            void main()
            {
                gl_Position = vec4(position.x, position.y, position.z, 1.0);
            }
        "#;
        const VERTEX_SHADER_FRAGMENT: &str = r#"
            #version 330 core
            out vec4 color;
            void main()
            {
                color = vec4(1.0f, 0.5f, 0.2f, 1.0f);
            }
        "#;
        let c_vertex_shader_source = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
        let c_vertex_shader_fragment = CString::new(VERTEX_SHADER_FRAGMENT.as_bytes()).unwrap();
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(
            vertex_shader,
            1,
            &c_vertex_shader_source.as_ptr(),
            ptr::null()
        );
        gl::CompileShader(vertex_shader);

        let mut success = gl::FALSE as GLint;
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut log_info = Vec::with_capacity(512);
            gl::GetShaderInfoLog(
                vertex_shader,
                512,
                ptr::null_mut(),
                log_info.as_mut_ptr() as *mut GLchar
            );
            println!("Vertex compilation failed\n{}", std::str::from_utf8(&log_info).unwrap());
        }

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(
            fragment_shader,
            1,
            &c_vertex_shader_fragment.as_ptr(),
            ptr::null()
        );
        gl::CompileShader(fragment_shader);
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut log_info = Vec::with_capacity(512);
            gl::GetShaderInfoLog(
                fragment_shader,
                512,
                ptr::null_mut(),
                log_info.as_mut_ptr() as *mut GLchar
            );
            println!("Fragment compilation failed\n{}", std::str::from_utf8(&log_info).unwrap());
        }

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut log_info = Vec::with_capacity(512);
            gl::GetShaderInfoLog(
                shader_program,
                512,
                ptr::null_mut(),
                log_info.as_mut_ptr() as *mut GLchar
            );
            println!("Program compilation failed\n{}", std::str::from_utf8(&log_info).unwrap());
        }

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        type Vertex = [GLfloat; 3];
        type Indexes = [u32; 3];

        let vertices: [Vertex; 4] = [
            [0.5,  0.5, 0.0],  // Верхний правый угол
            [0.5, -0.5, 0.0],  // Нижний правый угол
            [-0.5, -0.5, 0.0],  // Нижний левый угол
            [-0.5,  0.5, 0.0]   // Верхний левый угол
        ];
        let indices: [Indexes; 2] = [
          [0, 1, 3],
          [1, 2, 3]
        ];
        let (mut vbo, mut vao, mut ibo) = (0, 0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ibo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            mem::size_of_val(&vertices) as GLsizeiptr,
            vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            mem::size_of_val(&indices) as GLsizeiptr,
            indices.as_ptr() as *const c_void,
            gl::STATIC_DRAW
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>().try_into().unwrap(),
            ptr::null()
        );
        gl::EnableVertexAttribArray(0);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        (shader_program, vao)
    };


    while !window.should_close() {
        process_events(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);
            // gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            gl::BindVertexArray(0);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}

