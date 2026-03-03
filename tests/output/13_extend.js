import { __print } from './__util.js';
import * as __std_string from './std/string.js';
import * as __user from './__user_ext.js';
const u = { name: "Amihere", age: 30 };
const msg = __user._ext_User_greet(u);
__print(msg);
const s = "hello auwla";
__print(__user._ext_string_shout(s));
