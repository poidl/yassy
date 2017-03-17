#!/bin/python
# pylint: disable=C0103

"""Understand the postfilter for BLIT sawtooth VA described by Frei (p. 17)."""

# Frei, B.: Digital sound generation. Institute for Computer Music and
# Sound Technology (ICST) Zurich University of the Arts.

import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import bindings
import utils

b0 = 1.54
a1 = 0.54


bdy = 1.2
x = np.arange(-bdy, bdy, 0.01)
y = np.arange(-bdy, bdy, 0.01)
[xx, yy] = np.meshgrid(x, y)

fac = b0 / (xx**2 + yy**2 + 2 * a1 * xx + a1**2)
re = fac * (xx**2 + yy**2 + a1 * xx)
im = fac * a1 * yy

re[re < 0.5] = np.nan
re[re > 5] = np.nan

figname = 'h.svg'
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

#######################################

phi = np.arange(0.0, 3, 0.01)
p = b0 * (1 + a1 * np.cos(phi)) / (1 + 2 * a1 * np.cos(phi) + a1**2)

s = 1 / phi

figname = 'h_unit.svg'
fig = plt.figure()
plt.semilogy(phi / (2 * np.pi), p)
plt.semilogy(phi / (2 * np.pi), s)
plt.grid()
fig.savefig('../figures/' + figname)
print('done')

# ########################################
# # C_0 = 16.35
# # 44100 / C_0 = 2697.25
# # fs = 44800
# f0 = 8000
# # signal length in fs-sampling intervals
# N = 49
# fs = N * f0

# # number of sample points per sampling interval (uneven, so there is a
# # point centered in the fs-sampling interval)

# saw = utils.sawtooth(N)
# ###
# plt.plot(saw)
# saw = utils.blit_apply(saw, f0, 48000)
# # plt.plot(saw)
# ###
# # patch to even length
# # patched signal length
# NP = len(saw) + 1001
# utils.assertEven(NP)
# saw = np.concatenate([saw, np.zeros(NP - len(saw))])
# a = utils.rdtpsd(saw, (49 / N) * (NP / N))


# f = np.linspace(0, 0.5, NP / 2 + 1)

# plt.semilogy(fs * f, a)
# plt.semilogy(fs * f, a, "*")
# plt.xlim(0, 50e3)
# # plt.ylim(1e-5, 1)
# plt.ylim(1e-5, 1e1)
# # plt.ylim(1e-38, 1e1)
# plt.grid()
# fig.savefig('../figures/' + 'tmp.svg')

# domega = fs * np.diff(f)[0]
# integ_full = domega * np.sum(a[fs * f < fs])
# integ = domega * np.sum(a[fs * f < fs * (49 / N)])

# print("f0: " + str(f0))
# print("fs: " + str(fs))
# print("domega: " + str(domega))
# print("integ_full: " + str(integ_full))
# print("integ: " + str(integ))


# # ###########################
# # T = 3
# # # signal length in fr-sampling intervals (uneven, append zeros to make even
# # # before FFT)
# # NR = N * (T - 1) + 1
# # fr = (NR / N) * fs
# # # Dimensionless time (in fr-samples)
# # # t = np.arange(0, NR, 1)
# # f0 = fs / N

# # # sanity check
# # utils.assertEven(N)
# # utils.assertOdd(T)
# # utils.assertOdd(NR)

# # saw = utils.saw(NR)
# # utils.assertOdd(T)

# # # patch to even length
# # # patched signal length
# # NP = len(saw) + 1
# # utils.assertEven(NP)
# # saw = np.concatenate([saw, np.zeros(NP - len(saw))])
# # a = utils.rdtpsd(saw, 1 / T)

# # # Nyquist freq
# # tmp = np.arange(0, NP / 2 + 1, 1)
# # # Dimensionless frequency (in fr-samples**(-1))
# # f = tmp / NP
# # fnyq = fr * (NP / NR)

# # plt.semilogy(fnyq * f, a, '*')
# # plt.axvline(fs)
# # # plt.axvline(f0)
# # # plt.xlim(0, 52000)
# # # plt.ylim(1e-5, np.max(a))

# # # blit = bindings.blit_2t(41000 / T)
# # # print(blit[0])
# # # imid = int(len(saw) / 2)

# # # right[0: 2 * T - 1] = right[0: 2 * T - 1] + blit
# # # left = left[0:-1]
# # # left[-2 * T + 2:] = left[-2 * T + 2:] - blit[-1:0:-1]
# # # saw2 = np.concatenate([left, right])
# # # fft2 = np.fft.rfft(saw2)
# # # fft2[0] = fft2[0] / 50
# # # fft2[-1] = fft2[-1] / 50
# # # fft2[1:-1] = fft2[1:-1] / 25
# # # a2 = np.abs(fft2)
# # # plt.semilogy(a2)
# # # plt.plot(saw2)

# # plt.ylim(1e-5, 1)
# # plt.grid()
# # fig.savefig('../figures/' + 'tmp.svg')
