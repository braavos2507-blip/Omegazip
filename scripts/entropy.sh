#!/bin/sh
# Энтропия файла (0–1). Используется скриптами анализа.
# $1 = путь к файлу
perl -e '
use strict;
my $file = $ARGV[0];
open F, "<", $file or die;
binmode F;
my ($n, @cnt) = (0, (0) x 256);
while (read F, my $b, 1) { $cnt[ord($b)]++; $n++; }
close F;
my $h = 0;
for my $c (@cnt) {
  next if $c == 0;
  my $p = $c / $n;
  $h -= $p * log($p) / log(2);
}
print $h / 8;
' "$1"
