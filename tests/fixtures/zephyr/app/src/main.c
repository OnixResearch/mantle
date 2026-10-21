/* SPDX-License-Identifier: Apache-2.0 */
#include <zephyr/kernel.h>
#include <zephyr/devicetree.h>
#include <zephyr/sys/printk.h>

int main(void)
{
	printk("Mantle Zephyr fixture: uart=%u stack=%u\n",
	       (unsigned int)DT_PROP(DT_NODELABEL(uart0), current_speed),
	       (unsigned int)CONFIG_MAIN_STACK_SIZE);
	return 0;
}
