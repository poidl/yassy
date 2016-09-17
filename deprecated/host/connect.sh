killall yassyhost
sleep 0.2
./target/debug/yassyhost &
sleep 0.2
jack_connect "yassyhost:midi_in"  "a2j:VMini [20] (capture): VMini MIDI 1"
jack_connect "yassyhost:audio_out" "system:playback_1"
jack_connect "yassyhost:audio_out" "system:playback_2"
