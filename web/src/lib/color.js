const modTextColor = "black";

// basically stolen from qira
const writeFg = "#FFFF00"; // bright yellow
const readFg = "#888800"; // dark yellow

// returns "backrtgound: ..." or ""
export function rwCssEntry(read, write) {
    let left = "";
    let right = "";
    if (read && write) {
        left = readFg;
        right = writeFg;
    } else if (read) {
        left = readFg;
        right = readFg;
    } else if (write) {
        left = writeFg;
        right = writeFg;
    } else {
        return "";
    }

    return `background: linear-gradient(90deg, ${left} 15%, ${right} 85%); color: ${modTextColor}`;
}

const idxSelected = "#ff0000c0";
const addressSelected = "#ff000060";
export function idxCss(selected) {
    return selected ? `background-color: ${idxSelected};` : "";
}

export function adrCss(selected) {
    return selected ? `background-color: ${addressSelected};` : ""
}
