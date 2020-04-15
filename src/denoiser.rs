use crate::vec::Vec3;

#[link(name = "optix.6.5.0")]
extern "system" {
    fn rtContextCreate(context: *mut usize) -> u32;
    fn rtContextDestroy(context: usize) -> u32;
    fn rtContextValidate(context: usize) -> u32;
    fn rtContextCompile(context: usize) -> u32;
    fn rtBufferCreate(context: usize, desc: u32, buffer: *mut usize) -> u32;
    fn rtBufferDestroy(buffer: usize) -> u32;
    fn rtBufferSetFormat(buffer: usize, format: u32) -> u32;
    fn rtBufferSetSize2D(buffer: usize, width: u32, height: u32) -> u32;
    fn rtBufferMapEx(buffer: usize, map_flags: u32, level: u32, user_owned: usize, 
        optix_owned: *mut usize) -> u32;
    fn rtBufferUnmapEx(buffer: usize, level: u32) -> u32;
    fn rtPostProcessingStageCreateBuiltin(context: usize, name: *const u8, 
        stage: *mut usize) -> u32;
    fn rtPostProcessingStageDestroy(stage: usize) -> u32;
    fn rtCommandListCreate(context: usize, command_list: *mut usize) -> u32;
    fn rtCommandListDestroy(command_list: usize) -> u32;
    fn rtCommandListAppendPostprocessingStage(command_list: usize, stage: usize,
        width: u32, height: u32) -> u32;
    fn rtCommandListFinalize(command_list: usize) -> u32;
    fn rtCommandListExecute(command_list: usize) -> u32;
    fn rtPostProcessingStageDeclareVariable(stage: usize, name: *const u8, 
        variable: *mut usize) -> u32;
    fn rtVariableSetObject(variable: usize, object: usize) -> u32;
    fn rtVariableSet1ui(variable: usize, v: u32) -> u32;
    fn rtVariableSet1f(variable: usize, v: f32) -> u32;
}

const RT_BUFFER_INPUT_OUTPUT: u32 = 3;
const RT_BUFFER_MAP_READ_WRITE: u32 = 2;
const RT_FORMAT_FLOAT4: u32 = 0x104;

unsafe fn create_buffer(context: usize, width: u32, height: u32, format: u32) -> usize {
    let mut buffer = 0;
    assert_eq!(rtBufferCreate(context, RT_BUFFER_INPUT_OUTPUT, &mut buffer), 0);
    assert_eq!(rtBufferSetFormat(buffer, format), 0);
    assert_eq!(rtBufferSetSize2D(buffer, width, height), 0);

    buffer
}

unsafe fn map_buffer(buffer: usize) -> usize {
    let mut mapped = 0;
    assert_eq!(rtBufferMapEx(buffer, RT_BUFFER_MAP_READ_WRITE, 0, 0, &mut mapped), 0);

    mapped
}

unsafe fn unmap_buffer(buffer: usize) {
    assert_eq!(rtBufferUnmapEx(buffer, 0), 0);
}

unsafe fn declare_variable(stage: usize, name: &[u8]) -> usize {
    let mut variable = 0;
    assert_eq!(rtPostProcessingStageDeclareVariable(stage, name.as_ptr(), &mut variable), 0);

    variable
}

pub fn denoise(width: u32, height: u32, pixels: &mut [Vec3]) {
    let channel_count = 4;
    let pixel_count   = (width * height) as usize;
    
    assert_eq!(pixel_count, pixels.len());

    unsafe {
        let mut context = 0;
        assert_eq!(rtContextCreate(&mut context), 0);

        let input_buffer  = create_buffer(context, width, height, RT_FORMAT_FLOAT4);
        let output_buffer = create_buffer(context, width, height, RT_FORMAT_FLOAT4);

        {
            let mapped = map_buffer(input_buffer) as *mut f32;
            let mapped = std::slice::from_raw_parts_mut(mapped, pixel_count * channel_count);

            for (i, pixel) in pixels.iter().enumerate() {
                let slice = &mut mapped[i * channel_count..];

                slice[0] = pixel.x;
                slice[1] = pixel.y;
                slice[2] = pixel.z;
                slice[3] = 1.0;
            }

            unmap_buffer(input_buffer);
        }

        let mut postprocess = 0;
        assert_eq!(rtPostProcessingStageCreateBuiltin(context, b"DLDenoiser\0".as_ptr(),
            &mut postprocess), 0);

        let vinput_buffer  = declare_variable(postprocess, b"input_buffer\0");
        let voutput_buffer = declare_variable(postprocess, b"output_buffer\0");
        let vblend         = declare_variable(postprocess, b"blend\0");
        let vhdr           = declare_variable(postprocess, b"hdr\0");

        assert_eq!(rtVariableSetObject(vinput_buffer, input_buffer), 0);
        assert_eq!(rtVariableSetObject(voutput_buffer, output_buffer), 0);
        assert_eq!(rtVariableSet1f(vblend, 0.0), 0);
        assert_eq!(rtVariableSet1ui(vhdr, 1), 0);

        let mut command_list = 0;
        assert_eq!(rtCommandListCreate(context, &mut command_list), 0);
        assert_eq!(rtCommandListAppendPostprocessingStage(command_list, postprocess,
            width, height), 0);
        assert_eq!(rtCommandListFinalize(command_list), 0);

        assert_eq!(rtContextValidate(context), 0);
        assert_eq!(rtContextCompile(context), 0);

        assert_eq!(rtCommandListExecute(command_list), 0);

        {
            let mapped = map_buffer(output_buffer) as *const f32;
            let mapped = std::slice::from_raw_parts(mapped, pixel_count * channel_count);

            for i in 0..pixel_count {
                let data = &mapped[i * 4..];

                pixels[i] = Vec3::new(data[0], data[1], data[2]);
            }

            unmap_buffer(output_buffer);
        }

        assert_eq!(rtCommandListDestroy(command_list), 0);
        assert_eq!(rtPostProcessingStageDestroy(postprocess), 0);
        assert_eq!(rtBufferDestroy(output_buffer), 0);
        assert_eq!(rtBufferDestroy(input_buffer), 0);
        assert_eq!(rtContextDestroy(context), 0);
    }
}
