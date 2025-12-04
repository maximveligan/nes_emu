#version 330 core

in vec2 uv;

out vec4 frag;

uniform sampler2D tex;

void main() {
	frag = texture(tex, uv);
}
