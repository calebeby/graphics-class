#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults
precision highp float;

uniform sampler2D twoDTex;

in vec4 vs_color;
in vec2 vs_uv;
in vec3 vs_vertex;
in vec3 vs_normal;
out vec4 color;

uniform mat4 camera_transform;
uniform mat4 transform;
uniform vec3 light_position;
uniform vec3 camera_position;

void main(void) {

  vec3 base_color = vec3(0.0, 0.5, 1.0);
  vec3 L_ambient = base_color; // around the scene light color
  vec3 L_diffuse = base_color + vec3(0.3, 0.3, 0.3); // Scattered light color
  vec3 L_specular = vec3(0.1, 0.1, 0.1); // Color of shininess of object

  // Material properties
  float K_ambient = 0.5;   // Ambient reflection coeff.
  float K_diffuse = 1.0;   // Diffuse reflection coeff.
  float K_specular = 10.0; // Specular reflection coeff.
  float alpha = 10.0;      // Specular exponent (m_gls)

  vec3 point = vs_vertex;
  vec3 view = normalize(camera_position - point);
  // Vector of the light source
  vec3 L = normalize(light_position - point);
  // Attenuation factor based on distance from light source
  /* float F_attenuation = 1.0 / length(light_position - point); */
  // TODO: why did I have to disable this?
  float F_attenuation = 1.0;

  vec3 I;
  float costheta = dot(L, vs_normal);
  /* float costheta = max(dot(L, vs_normal), 0.0); // Diffuse reflection
     amount */
  // Remember that if the light is coming in at too much of an angle, there
  // will be no reflection (hence the 0.0) A different way to describe this
  // is:
  //    If the projection of the Light direction to the normal is negative,
  //    set diffuse amount to 0.0

  vec3 I_ambient = L_ambient * K_ambient; // Start by setting up ambient light
  // This is just a scalar for the 'room' light, general catch all for complex
  // reflections

  // If our reflectance angle is positive (or worth calculating)
  if (costheta > 0.0) {

    //               Light       Coefficient  Scale
    vec3 I_diffuse = L_diffuse * K_diffuse * costheta; // Diffuse component
    // The amount of diffuse light is scaled by the angle from the normal to the
    // light The more 'direct' the light, the brighter the surface This makes
    // sense if you think about rotating a cube, the more 'flat' the light is
    // the more 'bright' it will be
    //     the more 'parallel' or angled the light is, the lower the lighting

    vec3 R = (2.0 * vs_normal * costheta) - L; // Perfect reflectance vector
    // Remember R and L are part of the basic lighting model

    // Angle between the perfect reflectance and the
    // view direction Again if negative, cap to 0.
    /* float cosphi = max(dot(R, view), 0.0); */
    float cosphi = clamp(dot(R, view), 0.0, 1.0);

    // Specular component
    //                Light        Coeff        Scale
    vec3 I_specular = L_specular * K_specular * pow(cosphi, alpha);
    // Similar to diffuse, but more tightly bound to the angle between the pure
    // reflection and the view

    I = I_ambient + F_attenuation * (I_specular + I_diffuse);

  } else {
    // Angle is too great, just keep the ambient lighting componenet
    I = I_ambient;
    /* I = vec3(1.0, 0.0, 0.0); */
  }

  vec4 lit_color = color = vec4(I, 1.0);
  /* color = vs_color; */
  vec4 tex = texture(twoDTex, vs_uv * vec2(1.0, 1.0));
  color = mix(lit_color, tex, 1.0);
}
