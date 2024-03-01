
#ifndef PERIPH_LED_HEADER_GUARD
#define PERIPH_LED_HEADER_GUARD

/* size of stack area used by each thread */
#define LED_STACKSIZE 1024

/* scheduling priority used by each thread */
#define LED_PRIORITY 7

#define LED0_NODE DT_ALIAS(led0)
#define LED1_NODE DT_ALIAS(led1)

#if !DT_NODE_HAS_STATUS(LED0_NODE, okay)
#error "Unsupported board: led0 devicetree alias is not defined"
#endif

#if !DT_NODE_HAS_STATUS(LED1_NODE, okay)
#error "Unsupported board: led1 devicetree alias is not defined"
#endif

void blink0(void);
void blink1(void);

#endif
