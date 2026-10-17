# Blender to native-client experiment

Both isolated branches are named experiment/custom-model-pipeline. The server
base is f9e4433f2451235805f85fe379d351d095b7535b on agent5/tower-development.
The client base is1a12b75b05744be762bc18402ec8c158221382bd on
agent1/client-development-20261015, verified against the remote before changes.
The older active local client ff9cef17fd1ce3ec2ebf8d1181fee31411999c4c and active
server remain untouched. Eleven existing opt-in Tower gateway/no-activate
validation patches were ported onto the latest client in the isolated clone.

Blender executable: `C:/Program Files/Blender Foundation/Blender 5.2/blender.exe`.
Version5.2.2 LTS; build d13f752e3b9c. Background/factory-startup operation only.
The fully original generator produces an editable blend, embedded GLB, material
manifest and preview. There are552 unique quantized positions and1012 triangles.
Native bounds are X[-255,255],Y[-504,0],Z[-64,64], equivalent to approximately
3.9844×1×3.9375 tiles. The Blender geometry is3.98×1×3.93895 tiles.

Model1000000 and objects65000/65001 were independently checked against62194
original object definitions and61606 original model groups. The original maximum
object ID is62200 and maximum model ID61605. No original IDs are replaced.

The only private client generated source input restored is java-char-case.bin,
16702 bytes, SHA256 b8d2d188ac11616e32ea77cef3b95d0c8ba8bb241b5e4bf9e3cf61a11c82cae6.
It stays ignored and is not published. Existing vendor dependencies, the GNU
Rust toolchain and native Argon2 are read-only inputs. Build targets, temporary
files, profile storage, credentials and SQLite state stay inside the experiment.
No active Agent workspace, character database or authoritative branch is changed.

The isolated authoritative test server uses127.0.0.1:63225 and its own
custom-arch-poc state directory. It publishes ordinary location65000 at local
(54,40),rotation0 and(60,38),rotation1, with four normal footing locations.
Original-cache pillar7016 at(52,40) provides a scale/reference-object check.
The offline static sample uses authentic cache paving and the same package;
its authored camera and scene lifecycle differ from live gameplay.

The first manually provisioned disposable account was rejected with login13
because it lacked an initial logout marker. This was a test-setup defect, not
asset loading. The provisioner now uses Accounts::save to establish that marker,
and the actual live validation uses normal server registration of a fresh user.
The failed title capture/log is retained privately and excluded from successful
live-scene evidence. No protected saved account was altered to force login.

Original cache media remain local. Public source includes only original art,
converter/registration code, engineering documents and non-sensitive manifests.
The small original GLB/model are included; editable blend and rendered outputs
remain local. The package has no extracted geometry or cache textures.
