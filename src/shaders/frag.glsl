#version 150

out vec4 color;

in vec3 frag_pos;

uniform vec3 params[10];
uniform vec3 sun_direction;

vec3 HosekWilkie(float cos_theta, float gamma, float cos_gamma) {
	vec3 A = params[0];
	vec3 B = params[1];
	vec3 C = params[2];
	vec3 D = params[3];
	vec3 E = params[4];
	vec3 F = params[5];
	vec3 G = params[6];
	vec3 H = params[7];
	vec3 I = params[8];
	vec3 Z = params[9];
	vec3 chi = (1 + cos_gamma * cos_gamma) / pow(1 + H * H - 2 * cos_gamma * H, vec3(1.5));
    return (1 + A * exp(B / (cos_theta + 0.01))) * (C + D * exp(E * gamma) + F * (cos_gamma * cos_gamma) + G * chi + I * sqrt(cos_theta));
}

void main(void) {
	vec3 V = normalize(frag_pos);
	float cos_theta = clamp(V.y, 0, 1);
	float cos_gamma = dot(V, sun_direction);
	float gamma = acos(cos_gamma);

	vec3 Z = params[9];
	vec3 R = Z * HosekWilkie(cos_theta, gamma, cos_gamma);
	if (cos_gamma > 0) {
		// Only positive values of dot product, so we don't end up creating two
		// spots of light 180 degrees apart
		R = R + pow(vec3(cos_gamma), vec3(256)) * 0.5;
	}
	color = vec4(R, 1.0);
}
