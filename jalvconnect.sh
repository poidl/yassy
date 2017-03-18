killall jalv
sleep 0.2
jalv -s http://example.org/yassy &
sleep 0.2
# jack_connect "yassy (simple synth):in"  "a2j:Keystation 49 [20] (capture): Keystation 49 MIDI 1"
jack_connect "yassy (simple synth):in"  "a2j:VMini [24] (capture): VMini MIDI 1"
jack_connect "yassy (simple synth):out" "system:playback_1"
jack_connect "yassy (simple synth):out" "system:playback_2"
