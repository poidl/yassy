import ctypes
from ctypes import cdll


def blit_2t_py(fs):

    M = 2 * (2700 - 1) + 1
    class F64_M(ctypes.Structure):
        _fields_ = [("array", ctypes.c_double * M)]
    lib = cdll.LoadLibrary("../../target/debug/libyassy.so")
    lib.blit_2t.argtypes = [ctypes.c_double]
    lib.blit_2t.restype = F64_M
    temp = lib.blit_2t(fs)

    return temp.array[:]


def blit_2t(fs):
    return blit_2t_py(fs)
