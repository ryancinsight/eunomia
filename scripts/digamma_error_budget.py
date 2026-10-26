"""A priori error propagation for the finite digamma fixture domain.

Each pair is an exact-real center and an absolute error enclosure. No observed
implementation error is used to choose a bound. Primitive assumptions are one
ulp for log and two ulps for tan, verified independently at fixture arguments.
"""
import struct
import mpmath as mp


def budget(value, precision):
    u = mp.mpf(2) ** -precision
    eta = mp.mpf(2) ** (-149 if precision == 24 else -1074)
    x = mp.mpf(value)
    def quantize(v):
        if precision == 24:
            return struct.unpack('f', struct.pack('f', float(v)))[0]
        return float(v)
    current_native = quantize(1 - x) if x < 0 else value
    # This concrete constant rounds directly to the nearest native pi; assert
    # the numerator's relative-u premise despite the f64 source spelling.
    assert abs(mp.mpf(quantize(mp.pi))-mp.pi)/mp.pi <= u
    one = (mp.mpf(1), mp.mpf(0))

    def rounded(center, error):
        return center, error + u * (abs(center) + error) + eta / 2

    def add(a, b):
        return rounded(a[0] + b[0], a[1] + b[1])

    def multiply(a, b):
        center = a[0] * b[0]
        error = abs(a[0]) * b[1] + abs(b[0]) * a[1] + a[1] * b[1]
        return rounded(center, error)

    def reciprocal(a):
        center, error = a
        lower = abs(center) - error
        assert lower > 0
        return rounded(1 / center, error / (abs(center) * lower))

    def coefficient(numerator, denominator=1):
        center = mp.mpf(numerator) / denominator
        # Compile-time f64 rational division plus conversion into native T.
        # For f64 the conversion is identity; the extra allowance is explicit.
        return center, ((1+u)*(1+mp.mpf(2)**-53)-1) * abs(center)

    reflection = mp.mpf(0)
    if x < 0:
        # Nearest-integer remainder is exact by Sterbenz, including x<1/2.
        fraction = x - mp.nint(x)
        if abs(fraction) != mp.mpf('.5'):
            angle = mp.pi * fraction
            angle_error = (2*u + u*u) * abs(angle) + eta / 2
            lo, hi = angle - angle_error, angle + angle_error
            assert lo * hi > 0
            sine_min = min(abs(mp.sin(lo)), abs(mp.sin(hi)))
            # Mean-value bound for pi*cot(theta), then a direct relative
            # quotient enclosure (including all cross terms and 2-ulp tan's
            # subnormal absolute error). The finite fixture interval avoids 0.
            tangent_min = min(abs(mp.tan(lo)), abs(mp.tan(hi)))
            tangent_relative = (4*u + 2*eta/tangent_min)/(1-u)
            assert tangent_relative < 1
            quotient_relative = (1+u)**2/(1-tangent_relative)-1
            cotangent_max = max(abs(mp.pi/mp.tan(lo)), abs(mp.pi/mp.tan(hi)))
            reflection = mp.pi*angle_error/(sine_min*sine_min)
            reflection += quotient_relative*cotangent_max + eta/2
        current = add(one, (-x, mp.mpf(0)))
    else:
        current = x, mp.mpf(0)
    recurrence = mp.mpf(0), mp.mpf(0)
    # Loop count follows the representable branch for every shipped fixture;
    # threshold-adjacent fixtures check both sides independently.
    while current_native < 16:
        recurrence = add(recurrence, reciprocal(current))
        current = add(current, one)
        current_native = quantize(current_native + 1)
    inverse = reciprocal(current)
    square = multiply(inverse, inverse)
    coefficients = [(-3617,8160),(1,12),(-691,32760),(1,132),(-1,240),
                    (1,252),(-1,120),(1,12)]
    series = coefficient(*coefficients[0])
    for pair in coefficients[1:]:
        c = coefficient(*pair)
        # FMA rounds once after the exact product-plus-add.
        center = series[0]*square[0] + c[0]
        error = abs(series[0])*square[1] + abs(square[0])*series[1]
        error += series[1]*square[1] + c[1]
        series = rounded(center, error)
    logarithm = mp.log(current[0])
    # log's input conditioning plus its documented <1 ulp error.
    log_error = current[1]/(current[0]-current[1])
    log_error += 2*u*(abs(logarithm)+log_error) + eta/2
    half_inverse = multiply((mp.mpf('.5'),mp.mpf(0)),inverse)
    polynomial = multiply(square,series)
    leading = add((logarithm,log_error),(-half_inverse[0],half_inverse[1]))
    core = add(leading,(-polynomial[0],polynomial[1]))
    positive = add(core,(-recurrence[0],recurrence[1]))
    # DLMF's first-omitted-term bound, evaluated at the smallest argument.
    lower = current[0]-current[1]
    truncation = abs(mp.bernpoly(18,0))/(18*lower**18)
    # Reflection's final subtraction contributes a relative 1u term plus the
    # propagated input errors. Return A + r*|psi(x)|, with final fixture rounding.
    absolute = (positive[1] + reflection + truncation + eta)/(1-u)
    relative = 2*u/(1-u)
    # Round bounds outward into their serialized IEEE representation.
    import math
    return math.nextafter(float(absolute), math.inf), math.nextafter(float(relative), math.inf)
