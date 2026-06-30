# meewu/todo
## general/core
- [ ] add `meewu uninstall module.zip` (#1)
- [x] implement `sudo meewu` (#2)
- [ ] add `meewu version` (#3)
- [ ] add `meewu [disable/enable] module.zip` (#4)
- [ ] implement `meewu config`
- [ ] add `meewu registry`

## modules
- [x] consider meewu's base path
- [x] handle privileged modules (depends on core #2)

** **

# files
## modules/installer.rs
- [x] add SIP disabled check to `if self.manifest.metadata.requires_sip_off`
- [ ] add `if self.manifest.metadata.requires_reboot`
- [x] handle privileged modules (depends on core #2)