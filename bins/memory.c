#include <stdio.h>
#include <stdlib.h>

int arr[4];

int memory_stat() {
	arr[0] = 0xdead;
	arr[1] = 0xbeef;
	arr[2] = 0x1337;
	arr[3] = 0xbabe;
	
	return arr[2];
}

int memory_dyn() {
	int * dyn = (int*)malloc(sizeof(int) * 4);

	dyn[0] = 0xaaaa;
	dyn[1] = 0xbbbb;
	dyn[2] = 0xcccc;
	dyn[3] = 0xdddd;

	return dyn[2];
}

int main() {
	int s = memory_stat();
	printf("stat: %d\n", s);
	int d = memory_dyn();
	printf("dyn: %d\n", d);
	
	printf("arr[]: %p\n", &arr);
	void *p = NULL;
	printf("main: %p\n", (void*)main);
	printf("sp: %p\n", (void*)&p);
	printf("printf: %p\n", (void*)printf);
}
