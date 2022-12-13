# ylitalonrustu

Application which listens for MQTT channel and converts that data into socket binary format.

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