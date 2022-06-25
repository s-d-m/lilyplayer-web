#!/bin/bash

set -eux
set -o pipefail

THIS_DIR="$(dirname "${BASH_SOURCE[0]}")"

function make_single_html() {
    local readonly wasm_file="$1"
    local readonly output_file="$2"

    (
	cat "${THIS_DIR}/assets/lilyplayer.html.head"


	printf '%s' "	var emscriptenModuleSource = atob('"
	base64 --wrap=0 "${THIS_DIR}/assets/lilyplayer.js"
	printf '%s\n' "');"

	printf '%s' "	var data = _base64ToArrayBuffer('"
	base64 --wrap=0 "${wasm_file}"
	printf '%s\n' "');"

	cat "${THIS_DIR}/assets/lilyplayer.html.tail"

    ) > "${output_file}"
}

function usage() {
    printf '%s <options>\n' "$0"
    printf '\nOptions:'
    printf '\n	--wasm-file    path to web assembly file'
    printf '\n	--output-file  Where to save the html file with wasm embedded in it'
}

function main() {
    wasm_file=''
    output_file=''

    while [ $# -ge 1 ] ; do
	case $1 in
	    '--wasm-file')
		wasm_file="$2"
		shift 2
		;;
	    '--output-file')
		output_file="$2"
		shift 2
		;;
	    *)
		set +x
		(
		    usage
		    printf '\n\n\nUnknown parameter %s\n' "$1"
		) >&2

		return 3
		;;
	esac
    done

    if [ -z "$wasm_file" ] ; then
	set +x
	(
	    usage
	    printf '\n\n\n%s\n' '--wasm-file <path to input wasm file> is required'
	) >&2

	return 3
    fi


    if [ -z "$output_file" ] ; then
	set +x
	(
	    usage
	    printf '\n\n\n%s\n' '--output-file <path to output html file> is required'
	) >&2

	return 3
    fi

    make_single_html "${wasm_file}" "${output_file}"

}

main "$@"
