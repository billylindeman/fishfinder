# fishfinder


fishfinder is an ads-b software stack written in rust.  Currently it is a crude port of dump1090, but I have ambitions to implement a full ads-b in solution in rust that can speak the GDL90 protocol and interface with many EFB's.  The end goal is something similar to the stratux software stack, but fully implemented in rust and directly interfacing with the SDR hardware & sensors.  



## Dependencies

```
sudo apt install librtlsdr-dev rtl-sdr
```
