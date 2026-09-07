use std::num::{IntErrorKind};

pub fn compile_source(source: &str) -> Result<String, Vec<String>> {
	let lines = source.lines();
	let mut program = Vec::<Instruction>::new();

	let mut errors = Vec::<String>::new();

	for (i,line) in lines.enumerate() {
		let cleaned_line = line.trim().to_ascii_lowercase();

		// modify later to allow for inline comments
		if cleaned_line.starts_with(';') {
			continue;
		}

		let Some((instruction, args)) = cleaned_line.split_once(' ') else {
			continue;
		};

		match parse_instruction_archetype(instruction) {
			InstructionArchetype::Unknown => { errors.push(format!("Line {}: ERROR Unknown instruction", i)); continue; },
			InstructionArchetype::DestVal(opcode) => {
				let dest_val = match parse_dest_value(args) {
					Ok(pair) => pair,
					Err(err) => {
						errors.push(format!("Line {}: ERROR {}", i, err));
						continue;
					},
				};

				match dest_val {
					DestValuePair::Normal { flags} => {
						let instruction = Instruction::Normal { opcode: opcode | flags as u16 };

						program.push(instruction);
					},
					DestValuePair::Immediate { flags, immediate } => {
						let instruction = Instruction::Immediate { opcode: opcode | flags as u16, immediate };

						program.push(instruction);
					},
					DestValuePair::LinkedImmediate { flags, label } => {
						let instruction = Instruction::LinkedImmediate { opcode: opcode | flags as u16, label };

						program.push(instruction);
					},
				}
			},
		}
	}

	if !errors.is_empty() {
		return Err(errors);
	}

	let mut output = String::new();

	output.push('[');

	let empty_program = program.is_empty();

	for instruction in program {
		match instruction {
			Instruction::Normal { opcode } => {
				output.push_str(&opcode.to_string());
				output.push(',');
			},
			Instruction::Immediate { opcode, immediate } => {
				output.push_str(&opcode.to_string());
				output.push(',');
				output.push_str(&immediate.to_string());
				output.push(',');
			},
			Instruction::LinkedImmediate { opcode: _, label: _ } => {
				return Err(vec!["OH FUCK. THERES SOMEHOW A LINKEDIMMEDIATE IN THE OUTPUT PHASE!".to_string()]);
			}
		}
	}

	if !empty_program {
		output.pop();
	}
	
	output.push(']');

	Ok(output)
}

enum Register {
	Ax,
	Bx,
	Cx,
	Dx
}

impl Register {
	fn flags(&self) -> u8 {
		match self {
			Register::Ax => {
				0b00
			},
			Register::Bx => {
				0b01
			},
			Register::Cx => {
				0b10
			},
			Register::Dx => {
				0b11
			}
		}
	}
}

enum ArgumentType {
	Register(Register),
	Value(u16),
	Label(String),
	AddrRegister(Register),
	AddrValue(u16)
}

enum DestValuePair {
	Normal { flags: u8 },
	Immediate { flags: u8, immediate: u16 },
	LinkedImmediate { flags: u8, label: String }
}

enum InstructionArchetype {
	DestVal(u16),
	Unknown,
}

fn parse_instruction_archetype(instruction: &str) -> InstructionArchetype {
	match instruction {
		"mov" => InstructionArchetype::DestVal(0b0001_0000_1100_0000),
		"add" => InstructionArchetype::DestVal(0b0001_0001_1100_0000),
		"sub" => InstructionArchetype::DestVal(0b0001_0010_1100_0000),
		_ => InstructionArchetype::Unknown,
	}
}

/// Parses a Destination-value pair of args into their 6 bit representation.
fn parse_dest_value(args: &str) -> Result<DestValuePair, &str> {
	let args: Vec<&str> = args.split(',').map(|s| s.trim()).collect();

	if args.len() != 2 {
		return Result::Err("Invalid number of arguments");
	}

	let dest = match parse_argument(args[0]) {
		Ok(dest) => dest,
		Err(err) => {
			return Err(err);
		},
	};

	let value = match parse_argument(args[1]) {
		Ok(value) => value,
		Err(err) => {
			return Err(err);
		},
	};

	match (dest, value) {
		(ArgumentType::Register(dest_reg), ArgumentType::Register(val_reg)) => {
			let reg_flags = (val_reg.flags() << 2) | dest_reg.flags(); // rb_ra

			#[allow(clippy::identity_op)]
			let flags = (0b00 << 4) | reg_flags; // 00_rb_ra

			Result::Ok(DestValuePair::Normal { flags })
		},
		(ArgumentType::AddrRegister(dest_reg), ArgumentType::Register(val_reg)) => {
			let reg_flags = (val_reg.flags() << 2) | dest_reg.flags(); // rb_ra

			let flags = (0b01 << 4) | reg_flags; // 01_rb_ra

			Result::Ok(DestValuePair::Normal { flags })
		},
		(ArgumentType::Register(dest_reg), ArgumentType::AddrRegister(val_reg)) => {
			let reg_flags = (val_reg.flags() << 2) | dest_reg.flags(); // rb_ra

			let flags = (0b10 << 4) | reg_flags; // 10_rb_ra

			Result::Ok(DestValuePair::Normal { flags })
		},
		(ArgumentType::Register(dest_reg), ArgumentType::Value(val)) => {
			let reg_flags = dest_reg.flags(); // ra

			let flags = (0b1100 << 2) | reg_flags; // 1100_ra

			Result::Ok(DestValuePair::Immediate { flags, immediate: val })
		},
		(ArgumentType::Register(dest_reg), ArgumentType::Label(val)) => {
			let reg_flags = dest_reg.flags(); // ra

			let flags = (0b1100 << 2) | reg_flags; // 1100_ra

			Result::Ok(DestValuePair::LinkedImmediate { flags, label: val })
		},
		(ArgumentType::AddrRegister(dest_reg), ArgumentType::Value(val)) => {
			let reg_flags = dest_reg.flags(); // ra

			let flags = (0b1101 << 2) | reg_flags; // 1101_ra

			Result::Ok(DestValuePair::Immediate { flags, immediate: val })
		},
		(ArgumentType::AddrRegister(dest_reg), ArgumentType::Label(val)) => {
			let reg_flags = dest_reg.flags(); // ra

			let flags = (0b1101 << 2) | reg_flags; // 1101_ra

			Result::Ok(DestValuePair::LinkedImmediate { flags, label: val })
		},
		(ArgumentType::AddrValue(dest_val), ArgumentType::Register(val_reg)) => {
			let reg_flags = val_reg.flags(); // rb

			let flags = (0b1111 << 2) | reg_flags; // 1111_rb

			Result::Ok(DestValuePair::Immediate { flags, immediate: dest_val })
		},
		(ArgumentType::Register(dest_reg), ArgumentType::AddrValue(val)) => {
			let reg_flags = dest_reg.flags(); // ra

			let flags = (0b1110 << 2) | reg_flags; // 1110_ra

			Result::Ok(DestValuePair::Immediate { flags, immediate: val })
		},
		_ => {
			Result::Err("FUCK")
		}
	}
}

fn parse_argument(arg: &str) -> Result<ArgumentType, &str> {
	if is_mem_operand(arg) {
		let inner_value = arg.strip_prefix("[").expect("HOW did this happen").strip_suffix("]").expect("HOW did this happen");

		let parsed_inner_arg = match parse_argument(inner_value) {
			Ok(parsed_inner_arg) => parsed_inner_arg,
			Err(msg) => {
				return Err(msg);
			},
		};

		let parsed_arg = match parsed_inner_arg {
			ArgumentType::Register(reg) => {
				ArgumentType::AddrRegister(reg)
			},
			ArgumentType::Value(val) => {
				ArgumentType::AddrValue(val)
			},
			_ => {
				return Err("I am NOT implementing ts twin 😭🥀");
			}
		};

		return Ok(parsed_arg);
	}

	if is_register(arg) {
		let Ok(register) = parse_register(arg) else { return Err("Failed to parse argument"); };

		return Ok(ArgumentType::Register(register));
	}

	let parsed_arg = match parse_value(arg) {
		Ok(val) => val,
		Err(err) => {
			return Err(err);
		},
	};

	Ok(parsed_arg)
}

fn parse_value(arg: &str) -> Result<ArgumentType, &str> {
	let original_arg = arg;

	let (arg, radix) = match arg.strip_prefix("0x") {
		Some(arg) => (arg, 16),
		None => match arg.strip_prefix("0o") {
			Some(arg) => (arg, 8),
			None => match arg.strip_prefix("0b") {
				Some(arg) => (arg,2),
				None => (arg, 10)
			},
		},
	};

	let value = match u16::from_str_radix(arg, radix) {
		Ok(value) => value,
		Err(err) => {
			match err.kind() {
				IntErrorKind::PosOverflow => {
					return Err("Value is too large");
				},
				IntErrorKind::InvalidDigit => {
					if arg == original_arg {
						return Ok(ArgumentType::Label(arg.to_string()));
					}

					return Err("Failed to parse value");
				},
				_ => { 
					return Err("Failed to parse value");
				},
			}
		},
	};

	Ok(ArgumentType::Value(value))
}

fn parse_register(arg: &str) -> Result<Register, &str> {
	match arg {
		"ax" => Ok(Register::Ax),
		"bx" => Ok(Register::Bx),
		"cx" => Ok(Register::Cx),
		"dx" => Ok(Register::Dx),
		_ => Err("Failed to parse register")
	}
}

fn is_register(arg: &str) -> bool {
	matches!(arg, "ax" | "bx" | "cx" | "dx")
}

fn is_mem_operand(arg: &str) -> bool {
	arg.starts_with('[') && arg.ends_with(']')
}

enum Instruction {
	/// A normal 16 bit instruction
	Normal { opcode: u16 },
	/// An instruction word followed by an immediate value word
	Immediate { opcode: u16, immediate: u16 },
	/// Converted into an Immediate instruction by the linker
	LinkedImmediate { opcode: u16, label: String },
}