#!/bin/python
# pylint: disable=C0103

"""Understand the postfilter for BLIT sawtooth VA described by Frei (p. 17)."""

# Frei, B.: Digital sound generation. Institute for Computer Music and
# Sound Technology (ICST) Zurich University of the Arts.

import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

figname = 'h.svg'

b0 = 1.54
a1 = 0.54

bdy = 1.2
x = np.arange(-bdy, bdy, 0.01)
y = np.arange(-bdy, bdy, 0.01)
[xx, yy] = np.meshgrid(x, y)

fac = b0 / (xx**2 + yy**2 - 2 * a1 * xx + a1**2)
re = fac * (xx**2 + yy**2 - a1 * xx)
im = -(fac / a1) * yy

re[re < 0.5] = np.nan
re[re > 5] = np.nan
fig = plt.figure()
plt.contour(xx, yy, np.sqrt(xx ** 2 + yy ** 2), levels=[1.0])
plt.contourf(xx, yy, re)
plt.axhline(color='k')
plt.axvline(color='k')
plt.colorbar()
plt.xlabel('Re')
plt.ylabel('Im')

fig.savefig('../figures/' + figname)
print('done')

# phi = np.arange(0.0, 3, 0.01)
# p = a1 * (1 - b0 * np.cos(phi)) / (1 - 2 * b0 * np.cos(phi) + b0**2)

# s = 1 / phi
# plt.figure()
# plt.semilogy(phi / (2 * np.pi), p)
# plt.semilogy(phi / (2 * np.pi), s)
# plt.show()
