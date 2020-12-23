#!/bin/bash

set -eu -o pipefail

if [ -n "${V:-}" ]; then
    set -x
fi

CGROUP_BASE_MOUNT=/sys/fs/cgroup
CGROUPV1_MEMORY_MOUNT=$CGROUP_BASE_MOUNT/memory

help() {
    echo "usage:"
    echo " $(basename "$0") [-d <sample-int>] <cgroup>"
}

sample_intv=1

while getopts d:h name; do
    case "$name" in
        d)
            sample_intv="$OPTARG"
            ;;
        h)
            help
            exit 0
            ;;
        ?)
            echo "error: incorrect arguments"
            help
            exit 2
            ;;
    esac
done

shift $((OPTIND - 1))

cg="${1-}"
if [ -z "$cg" ]; then
    echo "error: missing cgroup"
    help
    exit 1
fi

while true; do
    total_rss="$(awk '/^rss / { print $2 }' < "$CGROUPV1_MEMORY_MOUNT/$cg/memory.stat")"
    echo "$(LC_ALL=C date --rfc-3339=ns),$total_rss"
    sleep "$sample_intv"
done
