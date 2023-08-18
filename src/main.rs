use i2c_linux::I2c;
use std::cell::RefCell;
use std::fs::File;

const I2C_BUS: &'static str = "/dev/i2c-10";
const I2C_ADDRESS: u16 = 0x2f;

const DRV_CONFIG_PORT: u8 = 0x20; // Configuration of alert, smbus, watchdog, clock
const FAN_STAT_PORT: u8 = 0x24; // Failue in watchdog, fan driver, spin or stall
const FAN_STALL_PORT: u8 = 0x25; // Indicates a Stall failuer
const FAN_SPIN_ST_PORT: u8 = 0x26; // Indicates a fail on spin up routine
const FAN_DRIVE_FAIL_PORT: u8 = 0x27; // Indicates which fan cannot reach the speed
const FAN_INTERRUPT_PORT: u8 = 0x29; // Mask or Unmask the error conditions
const FAN_PWM_DUTY_PORT: u8 = 0x2a; // Sets the value of FAN_SPEED_CTRL_PORT with 100% and 0% of duty cycle
const FAN_PWM_OUT_PORT: u8 = 0x2b; // Sets the output type, push-pull or open-drain
const FAN_PWM_FREQ45_PORT: u8 = 0x2c; // Sets the base frequency of the pwm outputs 4 and 5
const FAN_PWM_FREQ123_PORT: u8 = 0x2d; // Sets the base frequency of the pwm outputs 1, 2 and 3
const FAN_SPEED_CTRL_PORT: u8 = 0x30; // In manual mode set the fan speed, in auto mode only for read the fan speed
const FAN_PWM_FREQ_DIV1_1: u8 = 0x31; // Sets the number to divide the freq for driver 1
const FAN_PWM_FREQ_DIV1_2: u8 = 0x41; // Sets the number to divide the freq for driver 2
const FAN_PWM_FREQ_DIV1_3: u8 = 0x51; // Sets the number to divide the freq for driver 3
const FAN_PWM_FREQ_DIV1_4: u8 = 0x61; // Sets the number to divide the freq for driver 4
const FAN_PWM_FREQ_DIV1_5: u8 = 0x71; // Sets the number to divide the freq for driver 5
const FAN_CONFIG_1_1: u8 = 0x32; // Sets the operation mode of the fan 1
const FAN_CONFIG_2_1: u8 = 0x33; // Sets the operation mode of the fan 1
const FAN_CONFIG_1_2: u8 = 0x42; // Sets the operation mode of the fan 2
const FAN_CONFIG_2_2: u8 = 0x43; // Sets the operation mode of the fan 2
const FAN_CONFIG_1_3: u8 = 0x52; // Sets the operation mode of the fan 3
const FAN_CONFIG_2_3: u8 = 0x53; // Sets the operation mode of the fan 3
const FAN_CONFIG_1_4: u8 = 0x62; // Sets the operation mode of the fan 4
const FAN_CONFIG_2_4: u8 = 0x63; // Sets the operation mode of the fan 4
const FAN_CONFIG_1_5: u8 = 0x72; // Sets the operation mode of the fan 5
const FAN_CONFIG_2_5: u8 = 0x73; // Sets the operation mode of the fan 5
const FAN_PID_GAIN_1: u8 = 0x35; // Sets the curve for auto speed
const FAN_PID_GAIN_2: u8 = 0x45; // Sets the curve for auto speed
const FAN_PID_GAIN_3: u8 = 0x55; // Sets the curve for auto speed
const FAN_PID_GAIN_4: u8 = 0x65; // Sets the curve for auto speed
const FAN_PID_GAIN_5: u8 = 0x75; // Sets the curve for auto speed
const FAN_SPIN_CONF_1: u8 = 0x36; // Sets the spin routine for auto speed
const FAN_SPIN_CONF_2: u8 = 0x46; // Sets the spin routine for auto speed
const FAN_SPIN_CONF_3: u8 = 0x56; // Sets the spin routine for auto speed
const FAN_SPIN_CONF_4: u8 = 0x66; // Sets the spin routine for auto speed
const FAN_SPIN_CONF_5: u8 = 0x76; // Sets the spin routine for auto speed
const FAN_MAX_STEP_CONF_1: u8 = 0x37; // Sets the max step for auto speed
const FAN_MAX_STEP_CONF_2: u8 = 0x47; // Sets the max step for auto speed
const FAN_MAX_STEP_CONF_3: u8 = 0x57; // Sets the max step for auto speed
const FAN_MAX_STEP_CONF_4: u8 = 0x67; // Sets the max step for auto speed
const FAN_MAX_STEP_CONF_5: u8 = 0x77; // Sets the max step for auto speed
const FAN_MIN_DRV_CONF_1: u8 = 0x38; // Sets the min drive for auto speed
const FAN_MIN_DRV_CONF_2: u8 = 0x48; // Sets the min drive for auto speed
const FAN_MIN_DRV_CONF_3: u8 = 0x58; // Sets the min drive for auto speed
const FAN_MIN_DRV_CONF_4: u8 = 0x68; // Sets the min drive for auto speed
const FAN_MIN_DRV_CONF_5: u8 = 0x78; // Sets the min drive for auto speed
const FAN_TAC_LOW_PORT: u8 = 0x3f; // Actual Tac count reading
const FAN_TAC_HIGH_PORT: u8 = 0x3e;
const FAN_TAC_TGT_LOW: u8 = 0x3c; // Target Tac count
const FAN_TAC_TGT_HIGH: u8 = 0x3d;
const FAN_TAC_VALID_HIGH: u8 = 0x39; // Max tac reading
const FAN_TAC_FAIL_HIGH: u8 = 0x3b; // For checking not enough tac FAN_TAC > FAN_TAC_TGT - FAN_TAC_FAIL
const FAN_TAC_FAIL_LOW: u8 = 0x3a;

const FAN_TAC_HIGH_VAL: [u16; 8] = [4096, 2048, 1024, 512, 256, 128, 64, 32];
const FAN_TAC_LOW_VAL: [u16; 5] = [16, 8, 4, 2, 1];

pub struct Emc2301<'a> {
    path: &'a str,
    address: u16,
    i2c: RefCell<I2c<File>>,
}

impl<'a> Emc2301<'a> {
    fn new(path: &'a str, address: u16) -> Self {
        let mut i2c = I2c::from_path(path).unwrap();
        i2c.smbus_set_slave_address(I2C_ADDRESS, false).unwrap();
        let i2c = RefCell::new(i2c);
        Emc2301 { path, address, i2c }
    }

    fn address(&self) -> u16 {
        self.address
    }
    fn path(&self) -> &str {
        self.path
    }
    fn read_port(&self, port: u8) -> u8 {
        self.i2c.borrow_mut().smbus_read_byte_data(port).unwrap()
    }
    fn fan_speed(&self) -> u8 {
        self.read_port(FAN_SPEED_CTRL_PORT)
    }
    fn fan_status(&self) -> u8 {
        self.read_port(FAN_STAT_PORT)
    }
    fn fan_tac(&self) -> u16 {
        let mut tac: u16 = 0;
        let mut tac_low = self.read_port(FAN_TAC_LOW_PORT);
        let mut tac_high = self.read_port(FAN_TAC_HIGH_PORT);
        for x in FAN_TAC_HIGH_VAL {
            let bit = (tac_high & 0x80) == 0;
            tac_high = tac_high << 1;
            if bit {
                tac += x;
            }
        }

        for x in FAN_TAC_LOW_VAL {
            let bit = (tac_low & 0x80) == 0;
            tac_low = tac_low << 1;
            if bit {
                tac += x;
            }
        }
        tac
    }

    fn fan_tac_tgt(&self) -> u16 {
        let mut tac: u16 = 0;
        let mut tac_low = self.read_port(FAN_TAC_TGT_LOW);
        let mut tac_high = self.read_port(FAN_TAC_TGT_HIGH);
        for x in FAN_TAC_HIGH_VAL {
            let bit = (tac_high & 0x80) == 0;
            tac_high = tac_high << 1;
            if bit {
                tac += x;
            }
        }

        for x in FAN_TAC_LOW_VAL {
            let bit = (tac_low & 0x80) == 0;
            tac_low = tac_low << 1;
            if bit {
                tac += x;
            }
        }
        tac
    }
}

fn main() {
    let i2c = Emc2301::new(I2C_BUS, I2C_ADDRESS);
    println!(
        "Opened I2C on path {} and port {}",
        i2c.path(),
        i2c.address()
    );
    let data = i2c.fan_speed();
    println!("Fan speed: {:x}", data);
    let data = i2c.fan_tac();
    println!("Fan tach: {:x}", data);
    let data = i2c.fan_tac_tgt();
    println!("Fan tach target: {:x}", data);
}
