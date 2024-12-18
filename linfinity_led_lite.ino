#include <EEPROM.h>
#define LED_R 3
#define LED_G 5
#define LED_B 6

uint8_t mode = 2;
uint8_t speed = 2;
uint8_t r = 100;
uint8_t g = 0;
uint8_t b = 0;
bool on = true;

void setColor(int r, int g, int b) {
  analogWrite(LED_R, 255 - r);
  analogWrite(LED_G, 255 - g);
  analogWrite(LED_B, 255 - b);
}
void gayMode() {
  for (int i = 0; i <= 255; i++) {
    setColor(i, 255 - i, 0);
    delay(speed);
  }
  for (int i = 0; i <= 255; i++) {
    setColor(255 - i, 0, i);
    delay(speed);
  }
  for (int i = 0; i <= 255; i++) {
    setColor(0, i, 255 - i);
    delay(speed);
  }
}
void pulseMode() {
  for (int i = 0; i <= 255; i++) {
    setColor(i * r / 255, i * g / 255, i * b / 255);
    delay(speed);
  }
  for (int i = 255; i >= 0; i--) {
    setColor(i * r / 255, i * g / 255, i * b / 255);
    delay(speed);
  }
}
void setState() {
  EEPROM.write(0, mode);
  EEPROM.write(1, r);
  EEPROM.write(2, g);
  EEPROM.write(3, b);
  EEPROM.write(4, speed);
  Serial.println("OK");
  Serial.println(mode);
  Serial.println(r);
  Serial.println(g);
  Serial.println(b);
  Serial.println(speed);
}

void loadState() {
  mode = EEPROM.read(0);
  r = EEPROM.read(1);
  g = EEPROM.read(2);
  b = EEPROM.read(3);
  speed = EEPROM.read(4);
}

void setup() {
  Serial.begin(9600);
  pinMode(LED_R, OUTPUT);
  pinMode(LED_G, OUTPUT);
  pinMode(LED_B, OUTPUT);
  Serial.println("Initializing");
  loadState();
  Serial.println("Hello");
}

void loop() {
  if(Serial.available() >= 12) {
    String command = Serial.readString();
    mode = command.charAt(0) - 48;
    r = command.substring(1, 4).toInt();
    g = command.substring(4, 7).toInt();
    b = command.substring(7, 10).toInt();
    speed = command.substring(10, 13).toInt();
    setState();
  }
  if (on) {
    if (mode == 0) {
      setColor(r, g, b);
    } else if (mode == 1) {
      pulseMode();
    } else if (mode == 2) {
      gayMode();
    }
  } else {
    setColor(0, 0, 0);
  }
}
