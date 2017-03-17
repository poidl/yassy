#!/bin/python
# pylint: disable=C0103

"""Python translation of Frei Appendix B3."""

# Frei, B.: Digital sound generation. Institute for Computer Music and
# Sound Technology (ICST) Zurich University of the Arts.

import numpy as np
import scipy.signal as sig
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import utils

# parameters
fs = 48000
fc = 15000
rlen = 4
ppiv = 2700
beta = 8.3
apof = 0.5
apobeta = 0.5

# plotting parameters
# if subs2 is true, subsample to twice the sample rate, to get the x-axis
# limit up to fs. This is what Frei does, but doesn't this yield a wrong
# magnitude response if one uses the so-found coefficients in the actual
# sample rate. Better set to false for adjusting coefficients!
subs2 = False

pts = ppiv * rlen
x1 = np.arange(pts)
x2 = rlen * 2 * (x1 - (pts - 1) / 2 + 0.00001) / (pts - 1)
x3 = np.pi * fc / fs * x2
h = np.sin(x3) / x3
w = np.kaiser(pts, beta)
g = w * h

# apodization and normalization
aw = 1 - apof * np.kaiser(pts, apobeta)
g = aw * g
g = g / max(g)

# cumulative sum, normalization
g = np.cumsum(g)
g = 2.0 * g / g[-1]
g[int(np.floor(pts / 2)):] = g[int(np.floor(pts / 2)):] - 2.0
g = g / max(g)

fc = 12000
# number of sampling points relative to rlen, times ppiv
pts3 = (fs / fc) * ppiv
g2 = np.linspace(0, -2, pts3)
g2[int(np.floor(pts3 / 2)):] = g2[int(np.floor(pts3 / 2)):] + 2.0
# insert segment at istart
istart = int(np.floor(pts3 / 2) - ppiv * rlen / 2)
iend = istart + pts
g2[istart:iend] = g2[istart:iend] + g

if subs2:
    g2 = g2[::int(np.floor(ppiv / 2))]
else:
    g2 = g2[::int(np.floor(ppiv))]

wspec = np.abs(np.fft.rfft(g2, norm="ortho"))
wspec = wspec / max(wspec)


figname = 'frei_appendix_Fig_18_to_20.svg'
fig = plt.figure()

if subs2:
    zeroToOne = np.linspace(0, 1, len(wspec))
    xax = (fs / 1000) * zeroToOne
    xaxRad = 2 * np.pi * zeroToOne
else:
    zeroToOneHalf = np.linspace(0, 0.5, len(wspec))
    xax = (fs / 1000) * zeroToOneHalf
    xaxRad = 2 * np.pi * zeroToOneHalf

b0 = 1.54
a1 = 0.54
pf = b0 * (1 + a1 * np.cos(xaxRad)) / (1 + 2 * a1 * np.cos(xaxRad) + a1**2)

plt.semilogy(xax, wspec, "*")
plt.semilogy(xax, xax[1] / xax)
plt.semilogy(xax, wspec * pf, "*")
plt.grid()
# # markers at 20 kHz, fs - 20 kHz and fs
plt.axvline(fc / 1000, color="b")
plt.axvline(20, color="g")
plt.axvline(fs / 1000 - 20, color="r")
plt.axvline(fs / 1000, color="r")
plt.title('Magnitude Spectrum')
plt.xlabel('Frequency in kHz')
fig.savefig('../figures/' + figname)
