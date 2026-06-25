# Bound on the per-sample step of a sampled sine

    s[k]   = sin(φ[k])
    s[k+1] = sin(φ[k+1])
    Δφ = φ[k+1] − φ[k] ≤ 2π·f_max / f_s  (step)
    φ̄  = φ[k+1] + φ[k]                   (sum)

Write the two phases symmetrically about their midpoint:

    φ[k]   = φ̄/2 − Δφ/2
    φ[k+1] = φ̄/2 + Δφ/2

The even parts cancel and the odd parts add:

    s[k+1] − s[k] = 2·cos(φ̄/2)·sin(Δφ/2)

Taking absolute values:

    |s[k+1] − s[k]| = 2·|cos(φ̄/2)|·|sin(Δφ/2)|

    |cos(φ̄/2)| — local slope of the carrier at the midpoint
    |sin(Δφ/2)| — step-size factor

## Maximize the two factors

sin(Δφ/2) is monotonically increasing on Δφ/2 ∈ [0, π/2],
so it is maximized at the largest allowed step:

    |sin(Δφ/2)| ≤ sin(π·f_max / f_s)        valid only for f_max ≤ f_s/2

  Nyquist caveat: monotonicity needs Δφ/2 ≤ π/2 <-> Δφ_max ≤ π <-> f_max ≤ f_s/2.
  Past Nyquist sin(Δφ/2) turns over, the boundary stops being the maximizer,
  and the "phase step = instantaneous frequency" picture is aliased.

    |cos(φ̄/2)| ≤ 1,   attained when φ̄/2 ∈ {0, π, 2π, …}  <-> φ̄ ∈ {0, 2π, 4π, …}

This is the midpoint sitting on a zero crossing of the carrier — the steepest
point. (Note: it is φ̄/2, the midpoint, that hits {0, π, …}; the sum φ̄ is double
that.)

## The two maxima are simultaneously reachable

(φ̄, Δφ) is a linear, invertible map of (φ[k], φ[k+1]):

    ⎛ φ̄  ⎞   ⎛  1   1 ⎞ ⎛ φ[k]   ⎞
    ⎜    ⎟ = ⎜        ⎟ ⎜        ⎟ ,   det = 2 ≠ 0
    ⎝ Δφ ⎠   ⎝ −1   1 ⎠ ⎝ φ[k+1] ⎠

The constraint binds only Δφ and leaves φ̄ free, so both factors can be maxed at
once. The product of the maxima is therefore achievable.

## Result

    ┌─────────────────────────────────────────────────────────┐
    │ |s[k+1] − s[k]| ≤ 2·sin(π·f_max / f_s),   f_max ≤ f_s/2 │
    └─────────────────────────────────────────────────────────┘

### Worst-case configuration

Midpoint on an ascending zero crossing (φ̄ = 0) with the maximal step:

    φ[k]   = −π·f_max / f_s        s[k]   = −sin(π·f_max / f_s)
    φ[k+1] = +π·f_max / f_s        s[k+1] = +sin(π·f_max / f_s)

The samples straddle the crossing symmetrically; the jump spans the full
vertical chord across the steepest part of the wave.
