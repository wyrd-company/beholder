#include <stdint.h>

extern void bridgeCallback(uint32_t value, uintptr_t handle);

void bridge_invoke(uintptr_t handle) {
	bridgeCallback(7, handle);
}
