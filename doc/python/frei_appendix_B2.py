#!/bin/python
# pylint: disable=C0103

"""Python translation of Frei Appendix B1."""

# Frei, B.: Digital sound generation. Institute for Computer Music and
# Sound Technology (ICST) Zurich University of the Arts.

import numpy as np
import scipy.signal as sig
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
# import bindings
import utils

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
wspec = np.sqrt(np.abs(utils.rdtpsd(g2)))
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

fcomp = 21000
rng = 1 + 2 * np.floor(0.5 * rlen * zpad * fcomp / fs - 0.5)
xidx = np.arange(rng + 1)
a = wspec[: int(rng + 1)]
a = 1.0 / a
ftune = 0.35
f = xidx / (ftune + rlen * zpad)
wgt = np.arange((rng + 1) / 2,  0, -1)
wgt = 1 + wgt * wgt
# This is different to Frei's Matlab version. See Matlab/Scipy docs for
# differences in call signatures
aa = a[::2]
b = sig.remez(16, f, aa, wgt)
[w, h] = sig.freqz(b, 1, rlen * zpad, 'whole')

figname = 'frei_xy.svg'
fig = plt.figure()
plt.plot(fs * f / 1000, a)
plt.plot(0.5 * fs * w / np.pi / 1000, abs(h))
plt.xlim(0, fs / 1000)
plt.ylim(0, max(abs(h)))
plt.xlabel('Frequency in kHz')
plt.title('Prefilter Magnitude Response')
plt.grid()
fig.savefig('../figures/' + figname)


#  check by convolving prefilter and bandlimited impulse
imp = g[:ppiv:pts]
res = np.convolve(b, imp)
res = np.concatenate([res, np.zeros(1000 - len(res))])
wspec = np.sqrt(np.abs(utils.rdtpsd(res)))
rng = round(1000 * 20000 / fs)
xidx = np.arange(rng + 1)

figname = 'frei_xyz.svg'
fig = plt.figure()
plt.plot(20 * xidx / rng, wspec[: int(rng + 1)] / wspec[0])
plt.xlabel('Frequency in kHz')
plt.title('Normalized Overall Magnitude Response')
plt.grid()
fig.savefig('../figures/' + figname)
