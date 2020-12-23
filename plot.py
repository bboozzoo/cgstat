#!/usr/bin/env python3

import argparse
import datetime

import numpy as np
import matplotlib.pyplot as plt


def parse_arguments():
    parser = argparse.ArgumentParser(description="plot")
    parser.add_argument("data", metavar="FILE")
    parser.add_argument("-t", "--title", help="plot title", default="")
    parser.add_argument("-o", "--output", help="output plot to given file", default="")
    return parser.parse_args()


def main(opts):
    d = np.loadtxt(
        opts.data,
        delimiter=",",
        dtype={
            "names": ["sample_timestamp", "rss"],
            "formats": [datetime.datetime, np.float],
        },
    )

    mem_samples_MB = d["rss"] / 1024.0 / 1024.0
    min_val_MB = min(mem_samples_MB)
    max_val_MB = max(mem_samples_MB)
    plt.plot(mem_samples_MB, color="b", label="RSS")
    plt.grid()
    plt.grid(which="minor", linestyle="dotted")
    plt.minorticks_on()
    plt.ylabel("MB")
    plt.xlabel("sample")
    plt.axis([0, len(d), 0.9 * min_val_MB, 1.1 * max_val_MB])
    plt.hlines(
        max_val_MB, 0, len(d), color="r", label="max ({:.2f} MB)".format(max_val_MB)
    )
    plt.legend()
    if opts.title != "":
        plt.suptitle(opts.title)
    else:
        plt.suptitle(opts.data)

    if opts.output != "":
        plt.savefig(opts.output)
    else:
        plt.show()


if __name__ == "__main__":
    opts = parse_arguments()
    main(opts)
