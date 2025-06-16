ubuntu:

sudo apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
      gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
      gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
      gstreamer1.0-libav libgstrtspserver-1.0-dev libges-1.0-dev

need to remove the depencacy for sysprof-capture-4 from glib and gio

remove mount from gio

remove libdw from sudo nano /usr/lib/pkgconfig/gstreamer-1.0.pc
