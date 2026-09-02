// DuduClock BLE 蓝牙低功耗广播与数据服务 (no_std, VHCI direct)

use esp_println::println;
use esp_wifi::ble::controller::BleConnector;

unsafe extern "C" {
    fn API_vhci_host_check_send_available() -> bool;
    fn API_vhci_host_send_packet(data: *const u8, len: u16);
}

pub struct BleService<'a> {
    connector: BleConnector<'a>,
    is_advertising: bool,
}

impl<'a> BleService<'a> {
    pub fn new(connector: BleConnector<'a>) -> Self {
        Self {
            connector,
            is_advertising: false,
        }
    }

    /// 发送原始 HCI 数据包给 ESP32-C3 蓝牙硬件控制器
    pub fn send_hci(&self, packet: &[u8], delay: &esp_hal::delay::Delay) -> bool {
        unsafe {
            let mut retries = 0;
            while !API_vhci_host_check_send_available() && retries < 100 {
                delay.delay_millis(2);
                retries += 1;
            }

            if !API_vhci_host_check_send_available() {
                println!("[BLE] Controller not ready to accept HCI packet");
                return false;
            }

            API_vhci_host_send_packet(packet.as_ptr(), packet.len() as u16);
            delay.delay_millis(10);
        }
        true
    }

    /// 启动 BLE 蓝牙广播 (设备名称: Dudu-AI-Screen)
    pub fn start_advertising(&mut self, delay: &esp_hal::delay::Delay, device_name: &str) -> bool {
        println!("[BLE] Step 1: Sending HCI Reset...");
        // 1. 发送 HCI Reset 指令
        let hci_reset = [0x01, 0x03, 0x0C, 0x00];
        if !self.send_hci(&hci_reset, delay) {
            println!("[BLE] HCI Reset failed!");
            return false;
        }

        println!("[BLE] Step 2: Setting Adv Parameters...");
        // 2. 设置广播参数 (HCI_LE_Set_Advertising_Parameters: 100ms 间隔, ADV_IND 可连接广播)
        let adv_params: [u8; 19] = [
            0x01, 0x06, 0x20, 0x0F, // HCI Cmd header, Opcode 0x2006, Param Len = 15
            0xA0, 0x00, // Adv_Interval_Min = 100ms (160 * 0.625ms = 0x00A0)
            0xA0, 0x00, // Adv_Interval_Max = 100ms
            0x00,       // Adv_Type = 0x00 (ADV_IND)
            0x00,       // Own_Address_Type = Public (0x00)
            0x00,       // Peer_Address_Type = Public (0x00)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Peer_Address = 00:00:00:00:00:00
            0x07,       // Adv_Channel_Map = 0x07 (Channel 37, 38, 39)
            0x00,       // Adv_Filter_Policy = 0x00 (Allow scan from any, connect from any)
        ];
        if !self.send_hci(&adv_params, delay) {
            println!("[BLE] Set Adv Params failed!");
            return false;
        }

        println!("[BLE] Step 3: Setting Adv Data ('{}')...", device_name);
        // 3. 构建并设置广播数据包 (HCI_LE_Set_Advertising_Data)
        let name_bytes = device_name.as_bytes();
        let name_len = name_bytes.len().min(24);

        let mut adv_payload = [0u8; 31];
        adv_payload[0] = 0x02;
        adv_payload[1] = 0x01;
        adv_payload[2] = 0x06;

        let name_tag_len = (name_len + 1) as u8;
        adv_payload[3] = name_tag_len;
        adv_payload[4] = 0x09; // Complete Local Name
        for i in 0..name_len {
            adv_payload[5 + i] = name_bytes[i];
        }

        let total_data_len = 3 + (2 + name_len);

        let mut set_adv_data = [0u8; 36];
        set_adv_data[0] = 0x01;
        set_adv_data[1] = 0x08;
        set_adv_data[2] = 0x20;
        set_adv_data[3] = 0x20; // 32 bytes (1 len + 31 data)
        set_adv_data[4] = total_data_len as u8;
        set_adv_data[5..36].copy_from_slice(&adv_payload);

        if !self.send_hci(&set_adv_data, delay) {
            println!("[BLE] Set Adv Data failed!");
            return false;
        }

        println!("[BLE] Step 4: Enabling Advertising...");
        // 4. 开启广播 (HCI_LE_Set_Advertise_Enable = 0x01)
        let adv_enable = [0x01, 0x0A, 0x20, 0x01, 0x01];
        if !self.send_hci(&adv_enable, delay) {
            println!("[BLE] Enable Adv failed!");
            return false;
        }

        println!("[BLE] Bluetooth Advertising live & active: '{}'!", device_name);
        self.is_advertising = true;
        true
    }

    /// 轮询检查是否有来自蓝牙的数据包
    pub fn poll_packet(&mut self, buf: &mut [u8]) -> usize {
        self.connector.next(buf).unwrap_or(0)
    }

    pub fn is_advertising(&self) -> bool {
        self.is_advertising
    }
}
