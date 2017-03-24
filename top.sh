str=$(ps aux|grep ardour4|head -n 1|awk '{print $11}')
str=${str##\/*\/}
top -H -p `pidof $str`
