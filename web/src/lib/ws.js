import { writable } from "svelte/store";

export const sendStore = writable(null);
export const connectedStore = writable(false);

export const registerStore = writable(null);
export const memOpsStore = writable(null);
export const memoryStore = writable(null);
export const straceStore = writable(null);
export const stepStore = writable(null, () => {
    const socket = new WebSocket("ws://localhost:9001");

    socket.addEventListener("open", () => {
        connectedStore.set(true);
        sendStore.subscribe((msg) => { if (msg !== null) { socket.send(msg) } });
    });

    socket.addEventListener("message", (event) => {
        let msgs = JSON.parse(event.data);
        if (msgs.hasOwnProperty("steps")) {
            stepStore.set(msgs.steps);
        }
        if (msgs.hasOwnProperty("registers")) {
            registerStore.set(msgs.registers);
        }
        if (msgs.hasOwnProperty("mem_ops")) {
            memOpsStore.set(msgs.mem_ops);
        }
        if (msgs.hasOwnProperty("strace")) {
            straceStore.set(msgs.strace);
        }
        if (msgs.hasOwnProperty("memory")) {
            memoryStore.set(msgs.memory);
        }
    });


    return () => { console.log("CLOSING!"); connectedStore.set(false); socket.close() }
});
