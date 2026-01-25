/**
 * Masks the middle part of a string, keeping the first 4 and last 4 characters visible.
 * Used for serial numbers and UUIDs.
 */
export function maskString(str: string | null | undefined): string {
    if (!str) return "-";
    if (str.length <= 8) return str;
    return str.substring(0, 4) + "***" + str.substring(str.length - 4);
}

/**
 * Masks the beginning of a string, keeping the last 6 characters visible.
 * Used for device names.
 */
export function maskStart(str: string | null | undefined): string {
    if (!str) return "-";
    if (str.length <= 6) return "***" + str;
    return "***" + str.substring(str.length - 6);
}

/**
 * Masks the last segment of an IP address.
 */
export function maskIp(str: string | null | undefined): string {
    if (!str) return "-";
    const lastDot = str.lastIndexOf(".");
    if (lastDot !== -1) {
        return str.substring(0, lastDot) + ".***";
    }
    return str;
}
