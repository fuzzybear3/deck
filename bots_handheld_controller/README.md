ubuntu:

sudo apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
      gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
      gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
      gstreamer1.0-libav libgstrtspserver-1.0-dev libges-1.0-dev



#test sender command
gst-launch-1.0 -v \
  videotestsrc is-live=true \
    ! video/x-raw,width=640,height=480,framerate=30/1 \
    ! x264enc tune=zerolatency bitrate=1500 speed-preset=veryfast \
    ! rtph264pay config-interval=1 pt=96 \
    ! udpsink host=127.0.0.1 port=5000


#sender webcam
gst-launch-1.0 -v v4l2src device=/dev/video0 ! video/x-raw,width=640,height=480,framerate=30/1 ! videoconvert ! x264enc tune=zerolatency bitrate=1500 speed-preset=veryfast key-int-max=30 ! \
  rtph264pay config-interval=1 pt=96 ! \
  udpsink host=192.168.4.86 port=5000 sync=false
