#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults
precision highp float;

out vec4 vs_color;
out vec2 vs_uv;
out vec3 vs_vertex;
out vec3 vs_normal;

uniform mat4 camera_transform;
uniform mat4 transform;
uniform vec3 light_position;
uniform vec3 camera_position;
layout(location = 0) in vec4 obj_vertex;
layout(location = 1) in vec4 obj_normal;
layout(location = 2) in vec2 obj_uv;

void main(void) {
  gl_Position = camera_transform * transform * obj_vertex;

  vec3 normal = -normalize(mat3(transform) * vec3(obj_normal));
  vs_normal = normal;
  /* float shade_1 = clamp(dot(normal, vec3(-0.3, 0.3, -0.3)), 0.0, 1.0); */
  /* float shade_2 = clamp(dot(normal, vec3(0.0, 1.0, 0.0)), 0.0, 1.0); */
  /* float shade_3 = clamp(dot(normal, vec3(0.0, -1.0, 0.0)), 0.0, 1.0); */
  /* vs_color = */
  /*     vec4(vec3(0.7, 0.6, 0.5) * shade_1 + vec3(0.4, 0.4, 0.4) * shade_2
     + */
  /*              vec3(0.0, 0.0, 0.05) * shade_3 + vec3(0.05, 0.05, 0.05),
   */
  /*          1.0); */
  vs_uv = obj_uv;

  /* vec3 view = vec3(0.0, 0.0, 1.0); // Looking from the 'front' */
  vec3 view = normalize(camera_position);

  vec3 L_ambient = vec3(0.7, 0.5, 0.7);  // around the scene light color
  vec3 L_diffuse = vec3(0.7, 0.5, 0.7);  // Scattered light color
  vec3 L_specular = vec3(1.0, 1.0, 1.0); // Color of shininess of object

  // Material properties
  float K_ambient = 0.0; // Ambient reflection coeff.
  /* float K_diffuse = 0.0; // Diffuse reflection coeff. */
  float K_diffuse = 0.0;     // Diffuse reflection coeff.
  float K_specular = 1000.0; // Specular reflection coeff.
  float alpha = 200.0;       // Specular exponent (m_gls)

  vec3 point = vec3(transform * obj_vertex);
  // Vector of the light source
  vec3 L = normalize(light_position - point);
  // Attenuation factor based on distance from light source
  float F_attenuation = 1.0 / length(light_position - point);

  vec3 I;

  float costheta = max(dot(L, normal), 0.0); // Diffuse reflection amount
  // Remember that if the light is coming in at too much of an angle, there will
  // be no reflection (hence the 0.0) A different way to describe this is:
  //    If the projection of the Light direction to the normal is negative, set
  //    diffuse amount to 0.0

  vec3 I_ambient = L_ambient * K_ambient; // Start by setting up ambient light
  // This is just a scalar for the 'room' light, general catch all for complex
  // reflections

  int lMode = 0;

  // If our reflectance angle is positive (or worth calculating)
  if (costheta > 0.0) {

    //               Light       Coefficient  Scale
    vec3 I_diffuse = L_diffuse * K_diffuse * costheta; // Diffuse component
    // The amount of diffuse light is scaled by the angle from the normal to the
    // light The more 'direct' the light, the brighter the surface This makes
    // sense if you think about rotating a cube, the more 'flat' the light is
    // the more 'bright' it will be
    //     the more 'parallel' or angled the light is, the lower the lighting

    vec3 R = (2.0 * normal * costheta) - L; // Perfect reflectance vector
    // Remember R and L are part of the basic lighting model

    // Angle between the perfect reflectance and the
    // view direction Again if negative, cap to 0.
    float cosphi = max(dot(R, view), 0.0);

    //                Light        Coeff       Scale
    vec3 I_specular =
        L_specular * K_specular * pow(cosphi, alpha); // Specular component
    // Similar to diffuse, but more tightly bound to the angle between the pure
    // reflection and the view

    I = I_ambient + F_attenuation * (I_specular + I_diffuse);

  } else {
    // Angle is too great, just keep the ambient lighting componenet
    /* I = I_ambient; */
    I = vec3(0.0, 0.0, 0.0);
  }

  vs_color = vec4(I, 1.0);
  vs_vertex = point;
  /* vs_color = vec4(0.0, 0.0, 0.0, 1.0); */
}
