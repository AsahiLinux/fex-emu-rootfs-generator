#!/bin/sh

set -eu

layers=""
for f in /usr/share/fex-emu/layers/*; do
  [ -z "$layers" ] && layers=$f || layers="$layers $f"
done
dest="$1"

units=""
mounts=""
for layer in $layers; do
  name="$(basename $layer .erofs)"
  mount=/var/lib/fex-emu/layers/$name
  unit="$(systemd-escape $mount --path --suffix mount)"
  cat > $dest/$unit <<EOF
[Unit]
Description=FEX RootFS layer for $name

[Mount]
What=$layer
Where=$mount
EOF
  [ -z "$units" ] && units=$unit || units="$units $unit"
  [ -z "$mounts" ] && mounts=$mount || mounts="$mounts $mount"
done

rootfs=$(systemd-escape /var/lib/fex-emu/rootfs --path --suffix mount)
cat > $dest/$rootfs <<EOF
[Unit]
Description=FEX RootFS
BindsTo=$units
After=$units

[Mount]
What=overlay
Where=/var/lib/fex-emu/rootfs
Type=overlay
Options=lowerdir=$(echo $mounts|tr ' ' '\n' | tac | tr '\n' :|sed -e 's/.$//'),upperdir=/run/fex-emu/writable,workdir=/run/fex-emu/workdir

[Install]
WantedBy=multi-user.target
EOF

mkdir -p $dest/multi-user.target.wants/
ln -sf "../$rootfs" $dest/multi-user.target.wants/$rootfs
