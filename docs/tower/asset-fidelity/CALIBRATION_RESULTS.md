# Calibration results

The technical reference platform currently passes:

* 33 asset round trips / 165 matched native views, pixel-exact.
* 19 classic action sets / 335 matching native timestamps, pixel/coordinate/alpha exact.
* Independent transform tracer agreement at 335 native samples.
* Ten numeric/codec/socket/IK/registry boundary tests.
* Intentional Blender Python exception returns nonzero under `--python-exit-code 1`.

The last test matters: Blender's default process exit could remain zero after a
script exception. A missing local module initially caused an action-export run
to reuse older outputs. That failed run is retained privately; it is excluded
from accepted evidence. The wrapper now fails closed and checks expected action
coverage and every requested native timestamp. Matching only an available subset
is not accepted.

The 33-model and 19-action results are technical conversion evidence, not proof
of independent authoring. The previous goblin C is preserved and remains
unapproved. Independent component/static/choreography iteration results will be
recorded separately, including rejected regressions.
