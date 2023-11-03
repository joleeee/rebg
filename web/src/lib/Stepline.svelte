<script>
    import Step from "./Step.svelte";
    import { stepStore, sendStore, connectedStore } from "./ws.js";
    import { selectedAddress, selectedIdx, showSymbols } from "./stores";

    export let steps = [
        [0, 0, "0x0000005500806280", "this is fake data"],
        [0, 1, "0x0000005500806284", "str xzr, [sp]"],
        [0, 2, "0x0000005500806288", "cmp x0, #0x400"],
        [0, 3, "0x000000550080628c", "b.hs #0x55008064c8"],
        [0, 4, "0x0000005500806280", "sub sp, sp, x0"],
        [0, 5, "0x0000005500806284", "str xzr, [sp]"],
        [0, 6, "0x0000005500806288", "cmp x0, #0x400"],
        [0, 7, "0x000000550080629c", "cbnz x0, #0x55008062ac"],
        [0, 8, "0x00000055008062ac", "ldr x1, [x0, #0x28]"],
        [0, 9, "0x00000055008062b0", "cmp x1, x0"],
        [0, 10, "0x0000005500806280", "sub sp, sp, x0"],
        [0, 11, "0x0000005500806284", "str xzr, [sp]"],
        [0, 12, "0x0000005500806288", "cmp x0, #0x400"],
        [0, 13, "0x00000055008062c0", "ldr w1, [x0, #0x348]"],
        [0, 14, "0x00000055008062c4", "str x0, [x26, w22, uxtw #3]"],
        [0, 15, "0x00000055008062c8", "add w1, w1, #1"],
        [0, 16, "0x00000055008062cc", "str w1, [x0, #0x348]"],
        [0, 17, "0x00000055008062d0", "str w22, [x0, #0x41c]"],
        [0, 18, "0x00000055008062d4", "add w22, w22, #1"],
        [0, 19, "0x00000055008062d8", "ldr x0, [x0, #0x18]"],
        [0, 20, "0x00000055008062dc", "cbnz x0, #0x55008062ac"],
        [0, 21, "0x00000055008062e0", "cmp w23, w22"],
        [0, 22, "0x00000055008062e4", "cset w0, eq"],
        [0, 23, "0x00000055008062e8", "cmp x20, #0"],
        [0, 24, "0x00000055008062ec", "ccmp w0, #0, #0, eq"],
        [0, 25, "0x00000055008062f0", "b.eq #0x5500806530"],
        [0, 26, "0x00000055008062f4", "cmp x20, #0"],
        [0, 27, "0x00000055008062f8", "cset w2, eq"],
        [0, 28, "0x00000055008062fc", "cmp w0, #0"],
        [0, 29, "0x0000005500806300", "ccmp w2, #0, #0, eq"],
        [0, 30, "0x0000005500806304", "b.ne #0x5500806314"],
        [0, 31, "0x0000005500806314", "mov w1, w22"],
        [0, 32, "0x0000005500806318", "mov x0, x26"],
        [0, 33, "0x000000550080631c", "mov w3, #1"],
        [0, 34, "0x0000005500806320", "bl #0x5500811900"],
        [1, 35, "0x0000005500811854", "ldr x0, [x20, #0x400]"],
        [1, 36, "0x0000005500811858", "cbz x0, #0x5500811818"],
        [1, 37, "0x0000005500806324", "adrp x1, #0x550083c000"],
        [1, 38, "0x0000005500806328", "add x0, x21, #0xac8"],
        [1, 39, "0x000000550080632c", "mov x23, #0"],
        [1, 40, "0x0000005500806330", "ldr x1, [x1, #0xb28]"],
        [1, 41, "0x0000005500806334", "blr x1"],
        [2, 42, "0x0000005500806338", "cbz w22, #0x55008063dc"],
        [2, 43, "0x000000550080633c", "nop "],
        [2, 44, "0x0000005500806340", "ldr x28, [x26, x23, lsl #3]"],
        [2, 45, "0x0000005500806344", "ldrh w0, [x28, #0x34c]"],
        [2, 46, "0x0000005500806348", "tbz w0, #3, #0x55008063c4"],
        [2, 47, "0x000000550080634c", "ldr x1, [x28, #0x110]"],
        [2, 48, "0x0000005500806350", "and w0, w0, #0xfffffff7"],
        [2, 49, "0x0000005500806354", "strh w0, [x28, #0x34c]"],
        [2, 50, "0x0000005500806358", "cbz x1, #0x5500806434"],
        [2, 51, "0x000000550080635c", "adrp x0, #0x550083c000"],
        [2, 52, "0x0000005500806360", "ldr w0, [x0, #0xb60]"],
        [2, 53, "0x0000005500806364", "tbnz w0, #1, #0x5500806448"],
        [2, 54, "0x0000005500806368", "ldr x0, [x28, #0x120]"],
        [2, 55, "0x000000550080636c", "ldr x25, [x28]"],
        [2, 56, "0x0000005500806370", "ldr x0, [x0, #8]"],
        [2, 57, "0x0000005500806374", "ldr x2, [x1, #8]"],
        [2, 58, "0x0000005500806378", "lsr x1, x0, #3"],
        [2, 59, "0x000000550080637c", "sub w0, w1, #1"],
        [2, 60, "0x0000005500806380", "add x25, x25, x2"],
        [2, 61, "0x0000005500806384", "add x27, x25, w0, uxtw #3"],
        [2, 62, "0x0000005500806388", "cbz w1, #0x55008063a4"],
        [2, 63, "0x000000550080638c", "nop "],
        [2, 64, "0x0000005500806390", "ldr x1, [x27]"],
        [2, 65, "0x0000005500806394", "blr x1"],
        [3, 66, "0x00000000004005bc", "mov x29, sp"],
        [3, 67, "0x00000000004005c0", "str x19, [sp, #0x10]"],
        [3, 68, "0x00000000004005c4", "adrp x19, #0x411000"],
        [3, 69, "0x00000000004005c8", "ldrb w0, [x19, #0x38]"],
        [3, 70, "0x00000000004005cc", "cbnz w0, #0x4005dc"],
        [3, 71, "0x00000000004005d0", "bl #<call_weak_fn+20>"],
        [4, 72, "0x0000000000400550", "add x1, x0, #0x38"],
        [4, 73, "0x0000000000400554", "adrp x0, #0x411000"],
        [4, 74, "0x0000000000400558", "add x0, x0, #0x38"],
        [4, 75, "0x000000000040055c", "cmp x1, x0"],
        [4, 76, "0x0000000000400560", "b.eq #0x400578"],
        [4, 77, "0x0000000000400578", "ret "],
        [3, 78, "0x00000000004005d4", "mov w0, #1"],
        [3, 79, "0x00000000004005d8", "strb w0, [x19, #0x38]"],
        [3, 80, "0x00000000004005dc", "ldr x19, [sp, #0x10]"],
        [3, 81, "0x00000000004005e0", "ldp x29, x30, [sp], #0x20"],
        [3, 82, "0x00000000004005e4", "ret "],
        [2, 83, "0x0000005500806398", "cmp x25, x27"],
        [2, 84, "0x000000550080639c", "sub x27, x27, #8"],
        [2, 85, "0x00000055008063a0", "b.ne #0x5500806390"],
        [2, 86, "0x00000055008063a4", "ldr x0, [x28, #0xa8]"],
        [2, 87, "0x00000055008063a8", "cbz x0, #0x55008063bc"],
        [2, 88, "0x00000055008063ac", "ldr x1, [x28]"],
        [2, 89, "0x00000055008063b0", "ldr x0, [x0, #8]"],
        [2, 90, "0x00000055008063b4", "add x0, x1, x0"],
        [2, 91, "0x00000055008063b8", "blr x0"],
        [3, 92, "0x00000000004007c8", "mov x29, sp"],
        [3, 93, "0x00000000004007cc", "ldp x29, x30, [sp], #0x10"],
        [3, 94, "0x00000000004007d0", "ret "],
        [2, 95, "0x00000055008063bc", "mov x0, x28"],
        [2, 96, "0x00000055008063c0", "bl #0x5500815670"],
        [3, 97, "0x0000005500815670", "stp x29, x30, [sp, #-0x50]!"],
        [3, 98, "0x0000005500815674", "mov x29, sp"],
        [3, 99, "0x0000005500815678", "stp x19, x20, [sp, #0x10]"],
        [3, 100, "0x000000550081567c", "adrp x20, #0x550083c000"],
        [3, 101, "0x0000005500815680", "add x20, x20, #0xb60"],
        [3, 102, "0x0000005500815684", "mov x19, x0"],
        [3, 103, "0x0000005500815688", "ldr w0, [x20, #0x2b8]"],
        [3, 104, "0x000000550081568c", "cbnz w0, #0x550081569c"],
        [3, 105, "0x0000005500815690", "ldp x19, x20, [sp, #0x10]"],
        [3, 106, "0x0000005500815694", "ldp x29, x30, [sp], #0x50"],
        [3, 107, "0x0000005500815698", "ret "],
        [2, 108, "0x00000055008063c4", "ldr w0, [x28, #0x348]"],
        [2, 109, "0x00000055008063c8", "add x23, x23, #1"],
        [2, 110, "0x00000055008063cc", "sub w0, w0, #1"],
        [2, 111, "0x00000055008063d0", "str w0, [x28, #0x348]"],
        [2, 112, "0x00000055008063d4", "cmp w22, w23"],
        [2, 113, "0x00000055008063d8", "b.hi #0x5500806340"],
        [2, 114, "0x0000005500806434", "ldr x0, [x28, #0xa8]"],
        [2, 115, "0x0000005500806438", "cbz x0, #0x55008063bc"],
        [2, 116, "0x00000055008063dc", "mov x0, x20"],
        [2, 117, "0x00000055008063e0", "mov w1, #0"],
        [2, 118, "0x00000055008063e4", "bl #0x5500815470"],
        [3, 119, "0x00000055008063e8", "sub x20, x20, #1"],
        [3, 120, "0x00000055008063ec", "ldr x0, [x29, #0x80]"],
        [3, 121, "0x00000055008063f0", "sub x19, x19, #0xa8"],
        [3, 122, "0x00000055008063f4", "mov sp, x0"],
        [3, 123, "0x00000055008063f8", "cmn x20, #1"],
        [3, 124, "0x00000055008063fc", "b.ne #0x5500806210"],
        [3, 125, "0x0000005500806400", "ldr w0, [x29, #0x8c]"],
        [3, 126, "0x0000005500806404", "cbz w0, #0x550080646c"],
        [3, 127, "0x000000550080646c", "adrp x0, #0x550083c000"],
        [3, 128, "0x0000005500806470", "add x1, x0, #0xb60"],
        [3, 129, "0x0000005500806474", "ldr w1, [x1, #0x2b8]"],
        [3, 130, "0x0000005500806478", "cbz w1, #0x550080640c"],
        [3, 131, "0x000000550080640c", "ldr w0, [x0, #0xb60]"],
        [3, 132, "0x0000005500806410", "tbnz w0, #7, #0x55008064d0"],
        [3, 133, "0x0000005500806414", "mov sp, x29"],
        [3, 134, "0x0000005500806418", "ldp x19, x20, [sp, #0x10]"],
    ];

    let connected = false;
    connectedStore.subscribe((isConnected) => {
        connected = isConnected;
        if (!connected) {
            return;
        }
        steps = [];
    });
    stepStore.subscribe(recv_steps);

    function recv_steps(msgs) {
        if (msgs === null) {
            return;
        }
        msgs.forEach((step) => {
            let new_step = [step.d, step.i, step.a, step.c, step.s];
            steps = [...steps, new_step];
        });
    }

    let selected_idx = null;
    selectedIdx.subscribe((i) => (selected_idx = i));
    $: if (selected_idx != null && connected) {
        sendStore.set(JSON.stringify({ registers: selected_idx }));
        let from = Math.max(0, selected_adr - 4*8);
        sendStore.set(JSON.stringify({ memory: [from, 8, selected_idx] }));
    }

    let selected_adr = null;
    selectedAddress.subscribe((a) => (selected_adr = a));

    function step_selected(step) {
        selected_idx = step.detail.index;
    }

    let last_key = null;
    function key_press(event) {
        event.preventDefault();
        if (event.key === "Enter" && last_key !== null) {
            handle_action(last_key);
        } else {
            handle_action(event.key);
            last_key = event.key;
        }
    }

    function handle_action(action) {
        // this is important, js lmao (0 """is""" false)
        if (selected_idx === null) {
            // we want to be able to start moving around without using the mouse
            selected_idx = 0;
        }

        const prev = selected_idx;
        switch (action) {
            case " ":
                showSymbols.update((x) => !x);
                break;
            case "g":
                selected_idx = 0;
                break;
            case "G":
                selected_idx = steps.length - 1;
                break;
            case "j":
                selected_idx = Math.min(selected_idx + 1, steps.length - 1);
                break;
            case "k":
                selected_idx = Math.max(selected_idx - 1, 0);
                break;
            case "J": {
                let cur_idx = selected_idx;
                let target_depth = steps[selected_idx][0];
                // first just go down one
                cur_idx = Math.min(cur_idx + 1, steps.length - 1);
                // but then, continue if we're not at the same level
                while (
                    cur_idx < steps.length - 1 &&
                    steps[cur_idx][0] > target_depth
                ) {
                    cur_idx += 1;
                }
                // only jump if we actually found something on the same level (TODO figure out if we actually want this)
                if (steps[cur_idx][0] == target_depth) {
                    selected_idx = cur_idx;
                }
                break;
            }
            case "K": {
                let cur_idx = selected_idx;
                let target_depth = steps[selected_idx][0];
                cur_idx = Math.max(cur_idx - 1, 0);
                while (cur_idx > 0 && steps[cur_idx][0] > target_depth) {
                    cur_idx -= 1;
                }
                if (steps[cur_idx][0] == target_depth) {
                    selected_idx = cur_idx;
                }
                break;
            }
            case "h": {
                // @ts-ignore
                let target_depth = steps[selected_idx][0] - 1;
                let cur_idx = selected_idx;
                // @ts-ignore
                while (cur_idx > 0 && steps[cur_idx][0] > target_depth) {
                    cur_idx -= 1;
                }
                selected_idx = cur_idx;
                break;
            }
            case "l": {
                // @ts-ignore
                let target_depth = steps[selected_idx][0] + 1;
                // @ts-ignore
                let cur_idx = selected_idx;
                while (
                    cur_idx < steps.length - 1 &&
                    steps[cur_idx][0] == target_depth - 1 &&
                    steps[cur_idx + 1][0] >= steps[cur_idx][0]
                ) {
                    cur_idx += 1;
                }

                selected_idx = cur_idx;
                break;
            }
        }
        if (prev != selected_idx) {
            let step = steps[selected_idx];
            if (step[1] != selected_idx) {
                console.log("SHIT");
            }
            // hacky af
            selectedIdx.set(selected_idx);
            // selectedAddress.set(step[2]);
        }
    }
</script>

<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="outer" on:keypress={key_press} tabindex="0">
    {#each steps as step}
        <Step
            on:selected={step_selected}
            depth={step[0]}
            idx={step[1]}
            adr={step[2]}
            asm={step[3]}
            symbol={step[4]}
        />
    {/each}
</div>

<style>
    .outer {
        padding: 0;
        margin: 0;
    }
</style>
