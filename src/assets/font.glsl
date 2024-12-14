varying vec2 v_vt;

#ifdef VERTEX_SHADER
uniform ivec2 u_framebuffer_size;
uniform mat3 u_model_matrix;

attribute vec2 a_vt;
attribute vec2 a_pos;
void main() {
    v_vt = a_vt;
    vec3 pos = u_model_matrix * vec3(a_pos.x, float(u_framebuffer_size.y) - a_pos.y, 1.0);

    // from pixels to -1..1
    vec2 pos2 = pos.xy / pos.z;
    pos2 = 2.0 * pos2 / vec2(u_framebuffer_size);
    pos2 = vec2(pos2.x - 1.0, pos2.y - 1.0);

    gl_Position = vec4(pos2.xy, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec4 u_color;
uniform sampler2D u_cache_texture;
void main() {
    gl_FragColor = u_color;
    gl_FragColor.w *= texture2D(u_cache_texture, v_vt).w;
}
#endif
