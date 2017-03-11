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

# parameters
fs = 48000
fc = 15000
rlen = 4
ppiv = 2700
beta = 8.3
apof = 0.5
apobeta = 0.5

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

# cumulative sum, normalization
g = np.cumsum(g)
g = 2.0 * g / g[-1]
g[int(np.floor(pts / 2)):] = g[int(np.floor(pts / 2)):] - 2.0
g = g / max(g)


figname = 'frei_appendix_B3_BLIT_Fig_21.svg'
fig = plt.figure()
xax = np.linspace(0, rlen, pts)
plt.plot(xax, g)
fig.savefig('../figures/' + figname)
