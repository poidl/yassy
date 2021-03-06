#!/bin/python
# pylint: disable=C0103

"""Python translation of Frei Appendix B1 and B2."""

# Frei, B.: Digital sound generation. Institute for Computer Music and
# Sound Technology (ICST) Zurich University of the Arts.

import numpy as np
import scipy.signal as sig
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
# import bindings

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
x_save = (fmax / 1000) * xidx / rng
y_save = wspec[: (rng + 1)]
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
# This is different to Frei's Matlab version. See Matlab/Scipy docs for
# differences in call signatures. Whereas the Matlab function firpm
# (FIR Parks-McClellan, former "remez" wants amplitudes on the band edges,
# Scipy wants them in the band *center*
# As a consequence, the plots will differ from Frei Fig. 14
a = 0.5 * (wspec[: int(rng)] + wspec[1: int(rng + 1)])
a = a[::2]
a = 1.0 / a
ftune = 0.35
f = xidx / (ftune + rlen * zpad)
wgt = np.arange((rng + 1) / 2,  0, -1)
wgt = 1 + wgt * wgt

b = sig.remez(16, f, a, wgt)
[w, h] = sig.freqz(b, 1, rlen * zpad, 'whole')

figname = 'frei_prefilter_magnitude_response.svg'
fig = plt.figure()
f_center = 0.5 * (f[: -1] + f[1:])
f_center = f_center[::2]
plt.plot(fs * f_center / 1000, a)
plt.plot(0.5 * fs * w / np.pi / 1000, abs(h))
plt.xlim(0, fs / 1000)
plt.ylim(0, max(abs(h)))
plt.xlabel('Frequency in kHz')
plt.title('Prefilter Magnitude Response')
plt.grid()
fig.savefig('../figures/' + figname)


#  check by convolving prefilter and bandlimited impulse
imp = g[:pts:ppiv]
res = np.convolve(b, imp)
res = np.concatenate([res, np.zeros(1000 - len(res))])
wspec = np.abs(np.fft.rfft(res, norm="ortho"))
rng = round(1000 * 20000 / fs)
xidx = np.arange(rng + 1)

figname = 'frei_normalized_overall_magnitude_response.svg'
fig = plt.figure()
plt.plot(20 * xidx / rng, wspec[: int(rng + 1)] / wspec[0])
plt.xlim(0, 20)
plt.xlabel('Frequency in kHz')
plt.title('Normalized Overall Magnitude Response')
plt.grid()
fig.savefig('../figures/' + figname)

figname = 'frei_normalized_overall_magnitude_response_compare_1.svg'
fig = plt.figure()
plt.plot(20 * xidx / rng, wspec[: int(rng + 1)] / wspec[0])
plt.plot(x_save, y_save)
plt.xlim(0, 20)
plt.xlabel('Frequency in kHz')
plt.title('Normalized Overall Magnitude Response')
plt.grid()
fig.savefig('../figures/' + figname)

figname = 'frei_normalized_overall_magnitude_response_compare_2.svg'
fig = plt.figure()
xax = 0.5 * fs * w / np.pi / 1000
ynew = np.interp(xax, x_save, y_save)
plt.semilogy(xax, ynew)
plt.semilogy(xax, abs(h) * ynew)
plt.xlim(0, fs / 1000)
plt.ylim(1e-5, 1.1)
plt.xlabel('Frequency in kHz')
plt.title('Effect of prefilter')
plt.grid()
# markers at 20 kHz, fs - 20 kHz and fs
plt.axvline(20, color="g")
plt.axvline(fs / 1000 - 20, color="r")
plt.axvline(fs / 1000, color="r")
fig.savefig('../figures/' + figname)
