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

#  4 KHz = 48/12=48/(3*rlen) => n=3
n = 3
pts2 = ppiv * n * rlen
g2 = np.linspace(0, -2, pts2)
g2[int(np.floor(pts2 / 2)):] = g2[int(np.floor(pts2 / 2)):] + 2.0
# insert segment at istart
istart = int(np.floor(n / 2)) * ppiv * rlen
iend = int(np.floor(n / 2)) * ppiv * rlen + pts
g2[istart:iend] = g2[istart:iend] + g

# # subsample to sample rate
# g2 = g2[::int(np.floor(ppiv))]
# subsample to twice the sample rate (to get fs in the plot)
g2 = g2[::int(np.floor(ppiv / 2))]

wspec = np.abs(np.fft.rfft(g2, norm="ortho"))
wspec = wspec / max(wspec)


figname = 'frei_appendix_Fig_18.svg'
fig = plt.figure()
# plt.plot(g2, "*")
zeroToOne = np.linspace(0, 1, len(wspec))
xax = (fs / 1000) * zeroToOne
xaxRad = 2 * np.pi * zeroToOne
b0 = 1.54
a1 = 0.54
pf = b0 * (1 + a1 * np.cos(xaxRad)) / (1 + 2 * a1 * np.cos(xaxRad) + a1**2)

plt.semilogy(xax, wspec, "*")
plt.semilogy(xax, xax[1] / xax)
plt.semilogy(xax, wspec * pf, "*")
plt.grid()
# # markers at 20 kHz, fs - 20 kHz and fs
plt.axvline(fs / (3 * rlen * 1000), color="b")
plt.axvline(20, color="g")
plt.axvline(fs / 1000 - 20, color="r")
plt.axvline(fs / 1000, color="r")
plt.title('Magnitude Spectrum')
plt.xlabel('Frequency in kHz')
fig.savefig('../figures/' + figname)
