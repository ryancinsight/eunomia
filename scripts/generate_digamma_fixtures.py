"""Regenerate exact-binary real digamma fixtures with mpmath (100 digits)."""
from pathlib import Path
import math
import struct
import mpmath as mp
from digamma_error_budget import budget

mp.mp.dps = 100
points = [3.0, -2.5, -10.25, 1e-300, 1e-30, 1e-12, 1e-6, .125, .25, .5, 1., 2., 3.25, 8.,
          math.nextafter(16., 0.), 16., math.nextafter(16., math.inf), 32., 100.,
          1e6, 1e30, 1e300, -1e-300, -1e-30, -1e-12, -.125, -.25, -.5, -.75,
          -1.25, -1.5, -2.25, -10.5, -1e6-.25, math.nextafter(-1., 0.),
          math.nextafter(-1., -math.inf), 2.2250738585072014e-308,
          math.nextafter(2.2250738585072014e-308, 0.),
          -2.2250738585072014e-308,
          -math.nextafter(2.2250738585072014e-308, 0.)]
for root in [1.4616321449683623, -.5040830082644554, -1.5734984731623904,
             -2.6107208684441447]:
    points += [math.nextafter(root, -math.inf), root, math.nextafter(root, math.inf)]

# Exact rational grid avoids accidentally sampling only positive arguments.
points += [n / 16 for n in range(-255, 257, 17) if n % 16]

def single(value):
    return struct.unpack('f', struct.pack('f', value))[0]

singles = [single(x) for x in points if 1e-38 <= abs(x) <= 1e38
           and not (x < 0 and single(x).is_integer())]
singles += [single(1e-38), single(-1e-38), single(3.4028234663852886e38)]
for bits in [0x3fbb16c3, 0xbf010b1a, 0xbf800000]:
    singles += [struct.unpack('f', struct.pack('I', bits + offset))[0]
                for offset in [-1, 0, 1]
                if struct.unpack('f', struct.pack('I', bits + offset))[0] != -1.]

def literal(value):
    constants = {math.pi/4: 'core::f64::consts::FRAC_PI_4',
                 math.pi/8: 'core::f64::consts::FRAC_PI_8'}
    if abs(value) in constants:
        return ('-' if value < 0 else '') + constants[abs(value)]
    text = repr(value)
    mantissa, separator, exponent = text.partition('e')
    integer, dot, fraction = mantissa.partition('.')
    sign = '-' if integer.startswith('-') else ''
    integer = integer.lstrip('-')
    integer = sign + format(int(integer), '_')
    fraction = '_'.join(fraction[i:i+3] for i in range(0, len(fraction), 3))
    return integer + dot + fraction + separator + exponent

lines = ['//! Generated with mpmath 1.3.0, 100 decimal digits, exact binary inputs.']
for name, values in [('REFERENCES', points), ('SINGLE_REFERENCES', singles)]:
    lines.append(f'pub const {name}: &[(f64, f64)] = &[')
    for x in values:
        lines.append(f'    ({literal(x)}, {literal(float(mp.digamma(mp.mpf(x))))}),')
    lines.append('];')
directory = Path(__file__).resolve().parents[1] / 'crates/eunomia/tests/float_special'
(directory / 'digamma_fixtures.rs').write_text('\n'.join(lines)+'\n')
lines = ['//! A priori propagated budgets; see `scripts/digamma_error_budget.py`.']
for name, values, precision in [('BOUNDS', points, 53), ('SINGLE_BOUNDS', singles, 24)]:
    lines.append(f'pub const {name}: &[(f64, f64)] = &[')
    for x in values:
        absolute, relative = budget(x, precision)
        lines.append(f'    ({literal(absolute)}, {literal(relative)}),')
    lines.append('];')
(directory / 'digamma_error_bounds.rs').write_text('\n'.join(lines)+'\n')
# Primitive checks bind the quantified tan premise to actual rounded arguments.
lines = ['//! Primitive reference hi/lo values from exact binary arguments.']
for prefix, values, precision in [('', points, 53), ('SINGLE_', singles, 24)]:
    quantize = single if precision == 24 else float
    arguments = {'LOG': set(), 'TAN': set()}
    for x in values:
        current = quantize(1-mp.mpf(x)) if x < 0 else x
        while current < 16:
            current = quantize(current + 1)
        arguments['LOG'].add(current)
        if x < 0:
            fraction = x - round(x)
            if abs(fraction) != .5:
                arguments['TAN'].add(quantize(quantize(mp.pi)*mp.mpf(fraction)))
    for function, args in arguments.items():
        lines.append(f'pub const {prefix}{function}: &[(f64, f64, f64)] = &[')
        for x in sorted(args):
            expected = (mp.log if function=='LOG' else mp.tan)(mp.mpf(x))
            high = float(expected)
            low = float(expected-mp.mpf(high))
            lines.append(f'    ({literal(x)}, {literal(high)}, {literal(low)}),')
        lines.append('];')
(directory / 'digamma_primitive_fixtures.rs').write_text('\n'.join(lines)+'\n')

# Optional consumer fixture export contains data/bounds only, never scalar code.
import argparse
import json
parser = argparse.ArgumentParser()
parser.add_argument('--consumer-fixtures', type=Path)
arguments = parser.parse_args()
if arguments.consumer_fixtures:
    rows = []
    for x in points:
        absolute, relative = budget(x, 53)
        rows.append(dict(input=x, digamma=float(mp.digamma(mp.mpf(x))),
                         absolute=absolute, relative=relative))
    arguments.consumer_fixtures.parent.mkdir(parents=True, exist_ok=True)
    header = dict(generator='Eunomia scripts/generate_digamma_fixtures.py',
                  precision_digits=100, input_contract='exact binary64')
    content = json.dumps(header, indent=2)[:-2] + ',\n  "cases": [\n'
    content += ',\n'.join('    '+json.dumps(row) for row in rows)
    content += '\n  ]\n}\n'
    arguments.consumer_fixtures.write_text(content)
