# MQTT2Exmebus

Application which listens for MQTT channel and converts that data into socket binary format.

## Help

```
A piece of converter to convert incoming JSON into redisignal.

Usage: mqtt2exmebus [OPTIONS] --topic <TOPIC> --exmebus-port <EXMEBUS_PORT> --mqtt-port <MQTT_PORT> --host <HOST> --machine-id <MACHINE_ID> --mode <MODE>

Options:
      --topic <TOPIC>
          Topic of the MQTT-channel to listen ie. incoming/machine/HD453/json

      --exmebus-port <EXMEBUS_PORT>
          Exmebus-port where to forward data

      --mqtt-port <MQTT_PORT>
          Local mqtt port, 1883 or something

      --host <HOST>
          MQTT-server address ie. tcp://localhost

      --machine-id <MACHINE_ID>
          

      --mode <MODE>
          Mode of the parser json/redi

          Possible values:
          - json: Convert incoming data from JSON to redi
          - redi: Convert incoming data from redi signals to redi (not implemented)

  -d, --debug...
          Maximum debug level 2 ie. -dd

  -h, --help
          Print help information (use `-h` for a summary)

  -V, --version
          Print version information

```

## Testing
This in one console.
``` 
nc -l -p 124
```

This in second console.
``` 
./mqtt2exmebus --topic derp/ --exmebus-port 1234 --mqtt-port 1883 --host tcp://test.mosquitto.org --machine-id der123 --mode json -dd
```
And this in third console.
``` 
mosquitto_pub -h test.mosquitto.org -t derp/der123/json -m '{"id":5002,"ts":"13311953022000","dt":9,"val":"KISSUUUU"}' 

```