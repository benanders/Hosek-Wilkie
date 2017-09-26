#version 150

in vec3 position;
out vec3 frag_pos;

uniform mat4 projection;
uniform mat4 orientation;

void main(void) {
	frag_pos = normalize(position);
	gl_Position = projection * orientation * vec4(position, 1.0);
}
