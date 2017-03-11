#!/bin/python
# pylint: disable=C0103

"""Python translation of Frei Appendix B1."""

# Frei, B.: Digital sound generation. Institute for Computer Music and
# Sound Technology (ICST) Zurich University of the Arts.

import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

# parameters
fs = 48000
fc = 18300
rlen = 10
ppiv = 100
beta = 9
apof = 0.9
apobeta = 0.7

pts = ppiv * rlen + 1
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

# diagrams
figname = 'frei_appendixB1a.svg'
fig = plt.figure()
plt.plot(x2 / 2, g)
plt.xlim(-rlen / 2, rlen / 2)
plt.ylim(- 0.2, 1.0001)
plt.xlabel('Time in Sampling Intervals')
plt.title('Bandlimited Impulse')
plt.grid()
fig.savefig('../figures/' + figname)

zpad = 20
g2 = np.concatenate([g, np.zeros((zpad - 1) * pts)])
wspec = np.abs(np.fft.rfft(g2, norm="ortho"))
wspec = wspec / max(wspec)
# cut = 0.00001
# wspec[wspec > cut] = cut
fmax = 60000
rng = round(rlen * zpad * fmax / fs)
xidx = np.arange(rng + 1)

figname = 'frei_appendixB1b.svg'
fig = plt.figure()
plt.semilogy((fmax / 1000) * xidx / rng, wspec[: (rng + 1)])
plt.ylim(1e-5, 1)
plt.xlabel('Frequency in kHz')
plt.title('Amplitude Spectrum')
plt.grid()
# markers at 20 kHz, fs - 20 kHz and fs
plt.axvline(20, color="g")
plt.axvline(fs / 1000 - 20, color="r")
plt.axvline(fs / 1000, color="r")

fig.savefig('../figures/' + figname)
