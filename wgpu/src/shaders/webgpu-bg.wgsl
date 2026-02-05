 struct Uniforms {

    viewProjection: mat4x4f,

    viewPosition: vec3f,

    lightPosition: vec3f,

    shininess: f32,

  };


  @group(0) @binding(0) var<uniform> uni: Uniforms;


  struct Inst {

    mat: mat4x4f,

  };


  @group(0) @binding(1) var<storage, read> perInst: array<Inst>;


  struct VSInput {

      @location(0) position: vec4f,

      @location(1) normal: vec3f,

      @location(2) color: vec4f,

  };


  struct VSOutput {

    @builtin(position) position: vec4f,

    @location(0) normal: vec3f,

    @location(1) color: vec4f,

    @location(2) surfaceToLight: vec3f,

    @location(3) surfaceToView: vec3f,

  };


  @vertex

  fn myVSMain(v: VSInput, @builtin(instance_index) instanceIndex: u32) -> VSOutput {

    var vsOut: VSOutput;

    let world = perInst[instanceIndex].mat;

    vsOut.position = uni.viewProjection * world * v.position;

    vsOut.normal = (world * vec4f(v.normal, 0)).xyz;

    vsOut.color = v.color;


    let surfaceWorldPosition = (world * v.position).xyz;

    vsOut.surfaceToLight = uni.lightPosition - surfaceWorldPosition;

    vsOut.surfaceToView = uni.viewPosition - surfaceWorldPosition;


    return vsOut;

  }


  @fragment

  fn myFSMain(v: VSOutput) -> @location(0) vec4f {

    var normal = normalize(v.normal);


    let surfaceToLightDirection = normalize(v.surfaceToLight);

    let surfaceToViewDirection = normalize(v.surfaceToView);

    let halfVector = normalize(surfaceToLightDirection + surfaceToViewDirection);


    let light = dot(normal, surfaceToLightDirection) * 0.5 + 0.5;

 

    var specular = 0.0;

    if (light > 0.0) {

      specular = pow(dot(normal, halfVector), uni.shininess);

    }


    let out_color = vec3f(v.color.rgb * light + specular);

    return vec4f(pow(out_color, vec3f(2.2)), v.color.a);
  }