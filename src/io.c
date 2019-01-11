#include "kernel.h"

uint8_t inb(const uint16_t port)
{
	uint8_t ret;
	asm volatile("inb %1, %0" : "=a"(ret) : "d"(port));

	return ret;
}

void outb(const uint16_t port, const uint8_t value)
{
	asm volatile("outb %0, %1" : : "a"(value), "d"(port));
}