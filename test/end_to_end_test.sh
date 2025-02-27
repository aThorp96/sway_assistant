#!/usr/bin/env bash
set -euo pipefail

SWAYSOCK="$(mktemp -u).sock"
export SWAYSOCK

# Start headless sway
export DESKTOP_SESSION="sway"
export XDG_CURRENT_DESKTOP="sway"
export XDG_SESSION_DESKTOP="sway"
export XDG_SESSION_TYPE="wayland"
export WLR_BACKENDS="headless"
export WLR_LIBINPUT_NO_DEVICES=1
# Ensures sway initializes with outputs HEADLESS-1 and HEADLESS-2
export WLR_HEADLESS_OUTPUTS=2
sway 2>/dev/null &
echo "=> Sway session started with PID $!"
# Clean up the sway session on exit
trap "echo '=> cleaning sway session' && swaymsg exit || exit 0" EXIT

# Give sway time to start
sleep 1

# Setup monitors
swaymsg output HEADLESS-1 mode 1920x1080
swaymsg output HEADLESS-2 mode 2560x1600

echo "=> Setup outputs"
swaymsg -t get_outputs

function check_output_position() {
    output_name="${1}"
    expected_position="${2},${3}"

    outputs_json="$(swaymsg -t get_outputs --raw)"
    actual_position=$(jq -r ".[] | select(.name == \"${output_name}\") | \"\(.rect.x),\(.rect.y)\"" <(echo "${outputs_json}"))
    if [[ "${actual_position}" != "${expected_position}" ]]; then
        echo "Expected ${output_name} position to be ${expected_position} got ${actual_position}"
        return 1
    fi
}

function test_arrangement() {
    secondary_output="${1}"
    orientation="${2}"
    primary_output="${3}"
    secondary_x="${4}"
    secondary_y="${5}"
    primary_x="${6}"
    primary_y="${7}"

    echo "---"
    echo "=> Test: ${secondary_output} ${orientation} ${primary_output}"
    sway_assistant -s "${secondary_output}" "${orientation}" -p "${primary_output}" | sed 's/^/  -- /'
    check_output_position "${primary_output}" "${primary_x}" "${primary_y}"
    check_output_position "${secondary_output}" "${secondary_x}" "${secondary_y}"
    echo -e "=> Success"
}

# Check centering on above
test_arrangement HEADLESS-1 above HEADLESS-2  320    0    0 1080
test_arrangement HEADLESS-2 above HEADLESS-1    0    0  320 1600

# Check centering on below
test_arrangement HEADLESS-1 below HEADLESS-2  320 1600    0    0
test_arrangement HEADLESS-2 below HEADLESS-1  0   1080  320    0

# Check centering on left
test_arrangement HEADLESS-1 left HEADLESS-2    0  260 1920     0
test_arrangement HEADLESS-2 left HEADLESS-1    0    0 2560   260

# Check centering on right
test_arrangement HEADLESS-1 right HEADLESS-2 2560   260    0    0
test_arrangement HEADLESS-2 right HEADLESS-1 1920     0    0  260
