#!/bin/bash
sshpass -p "hypefuse" scp -r data hypefuse@linfinitysmartmirror.local:/hypefuse
curl http://linfinitysmartmirror.local:1337/trigger_reload_system >> /dev/null
