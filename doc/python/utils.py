#!/bin/python
# pylint: disable=C0103

"""DSP helper functions"""

import numpy as np
import bindings


def assertEven(x):
    """Raise exception if x is uneven"""
    if np.mod(x, 2) != 0:
        raise Exception("Length of signal is uneven")


def assertOdd(x):
    """Raise exception if x is even"""
    if np.mod(x, 2) == 0:
        raise Exception("Length of signal is even")


def rdtpsd(x: "sampled signal", dt: "intervals per sample" = 1):
    """Real discrete time power spectral density

    Keyword arguments:
    dt - - intervals per sample
    x - - sampled signal
    """
    assertEven(len(x))
    fft = np.fft.rfft(x)
    N = len(x)
    a = (dt / N) * (np.abs(fft)**2)
    return a


def sawtooth(N: "odd integer"):
    """Returns a sawtooth shape."""
    # sawtooth shape with odd len(), zero in center point
    assertOdd(N)
    tmp = np.linspace(0, -2, N)
    left = tmp[:int(np.floor(N / 2))]
    return np.concatenate([left, [0], -1 * left[::-1]])


def sawtooth_even(N: "even integer"):
    """Returns a sawtooth shape."""
    # sawtooth shape with even len()
    assertEven(N)
    tmp = np.linspace(0, -2, N)
    left = tmp[:int(N / 2)]
    return np.concatenate([left, -1 * left[::-1]])


def blit_apply(x: "signal of odd length", f0: "fundamental frequency", fs: "sampling frequency"):
    """Applies BLIT segment to sawtooth"""
    blit = bindings.blit_2t(fs)
    blit = np.array(blit)
    N = len(x)
    assertOdd(N)
    imid = int(np.floor(N / 2))
    fr = N * f0
    ni = int(np.floor(fr / (f0 * 2)))
    ic = np.arange(imid - ni, imid + ni + 1)
    lb = len(blit)
    idx = np.arange(0, lb, int(np.floor(lb / ni)))[1:]
    x[ic[ni + 1:]] = x[ic[ni + 1:]] + blit[idx]
    x[ic[:ni]] = -x[ic[:ni:-1]]
    return x
    # print("done")
