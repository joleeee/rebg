import { writable } from "svelte/store";

export const sendStore = writable();
export const connected = writable(false);

export const registerStore = writable(null);
export const stepStore = writable(null, () => {
    const socket = new WebSocket("ws://localhost:9001");

    socket.addEventListener("open", () => {
        connected.set(true);
    });

    socket.addEventListener("message", (event) => {
        let msgs = JSON.parse(event.data);
        if (msgs.hasOwnProperty("steps")) {
            stepStore.set(msgs.steps);
        }
        if (msgs.hasOwnProperty("registers")) {
            registerStore.set(msgs.registers);
        }
    });

    sendStore.subscribe((msg) => { /*socket.send(msg)*/ });

    return () => { connected.set(false); socket.close() }
});
