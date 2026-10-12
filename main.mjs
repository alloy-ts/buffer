import { createRequire } from "module";
//#region \0rolldown/runtime.js
var __defProp = Object.defineProperty;
var __exportAll = (all, no_symbols) => {
	let target = {};
	for (var name in all) __defProp(target, name, {
		get: all[name],
		enumerable: true
	});
	if (!no_symbols) __defProp(target, Symbol.toStringTag, { value: "Module" });
	return target;
};
//#endregion
//#region index.js
var app_exports = /* @__PURE__ */ __exportAll({
	Blob: () => Blob$1,
	Bytes: () => Bytes$1,
	File: () => File$1,
	__napiBindingTarget: () => __napiBindingTarget,
	add: () => add,
	atob: () => atob$1,
	btoa: () => btoa$1,
	bufByteLength: () => bufByteLength,
	bufCompare: () => bufCompare,
	bufConcat: () => bufConcat,
	bufCopyBytesFrom: () => bufCopyBytesFrom,
	bufDecode: () => bufDecode,
	bufEncode: () => bufEncode,
	bufFill: () => bufFill,
	bufIncludes: () => bufIncludes,
	bufIndexOf: () => bufIndexOf,
	bufIsEncoding: () => bufIsEncoding,
	bufLastIndexOf: () => bufLastIndexOf,
	bufStringLength: () => bufStringLength,
	bufWrite: () => bufWrite,
	isAscii: () => isAscii$1,
	isLatin1: () => isLatin1,
	isUtf8: () => isUtf8$1,
	resolveObjectURL: () => resolveObjectURL$1,
	transcode: () => transcode$1
});
const require = createRequire(import.meta.url);
const { readFileSync } = require("fs");
let nativeBinding = null;
let __napiLoadedBindingTarget = "native";
const loadErrors = [];
const isMusl = () => {
	let musl = false;
	if (process.platform === "linux") {
		musl = isMuslFromFilesystem();
		if (musl === null) musl = isMuslFromReport();
		if (musl === null) musl = isMuslFromChildProcess();
	}
	return musl;
};
const isFileMusl = (f) => f.includes("libc.musl-") || f.includes("ld-musl-");
const isMuslFromFilesystem = () => {
	try {
		return readFileSync("/usr/bin/ldd", "utf-8").includes("musl");
	} catch {
		return null;
	}
};
const isMuslFromReport = () => {
	let report = null;
	if (process.report && typeof process.report.getReport === "function") {
		process.report.excludeNetwork = true;
		report = process.report.getReport();
	}
	if (!report) return null;
	if (report.header && report.header.glibcVersionRuntime) return false;
	if (Array.isArray(report.sharedObjects)) {
		if (report.sharedObjects.some(isFileMusl)) return true;
	}
	return false;
};
const isMuslFromChildProcess = () => {
	try {
		return require("child_process").execSync("ldd --version", { encoding: "utf8" }).includes("musl");
	} catch (e) {
		return false;
	}
};
function requireNative() {
	if (process.env.NAPI_RS_NATIVE_LIBRARY_PATH) try {
		const overrideBinding = require(process.env.NAPI_RS_NATIVE_LIBRARY_PATH);
		__napiLoadedBindingTarget = overrideBinding && typeof overrideBinding.__napiBindingTarget === "string" ? overrideBinding.__napiBindingTarget : "native";
		return overrideBinding;
	} catch (err) {
		loadErrors.push(err);
	}
	else if (process.platform === "android") {
		if (process.arch === "arm64") {
			try {
				return require("./my-addon.android-arm64.node");
			} catch (e) {
				loadErrors.push(e);
			}
			try {
				const binding = require("@alloy-ts/buffer-android-arm64");
				const bindingPackageVersion = require("@alloy-ts/buffer-android-arm64/package.json").version;
				if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
				return binding;
			} catch (e) {
				loadErrors.push(e);
			}
		} else if (process.arch === "arm") {
			try {
				return require("./my-addon.android-arm-eabi.node");
			} catch (e) {
				loadErrors.push(e);
			}
			try {
				const binding = require("@alloy-ts/buffer-android-arm-eabi");
				const bindingPackageVersion = require("@alloy-ts/buffer-android-arm-eabi/package.json").version;
				if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
				return binding;
			} catch (e) {
				loadErrors.push(e);
			}
		} else loadErrors.push(/* @__PURE__ */ new Error(`Unsupported architecture on Android ${process.arch}`));
	} else if (process.platform === "win32") {
		if (process.arch === "x64") {
			if (process.config && process.config.variables && process.config.variables.shlib_suffix === "dll.a" || process.config && process.config.variables && process.config.variables.node_target_type === "shared_library") {
				try {
					return require("./my-addon.win32-x64-gnu.node");
				} catch (e) {
					loadErrors.push(e);
				}
				try {
					const binding = require("@alloy-ts/buffer-win32-x64-gnu");
					const bindingPackageVersion = require("@alloy-ts/buffer-win32-x64-gnu/package.json").version;
					if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
					return binding;
				} catch (e) {
					loadErrors.push(e);
				}
			} else {
				try {
					return require("./my-addon.win32-x64-msvc.node");
				} catch (e) {
					loadErrors.push(e);
				}
				try {
					const binding = require("@alloy-ts/buffer-win32-x64-msvc");
					const bindingPackageVersion = require("@alloy-ts/buffer-win32-x64-msvc/package.json").version;
					if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
					return binding;
				} catch (e) {
					loadErrors.push(e);
				}
			}
		} else if (process.arch === "ia32") {
			try {
				return require("./my-addon.win32-ia32-msvc.node");
			} catch (e) {
				loadErrors.push(e);
			}
			try {
				const binding = require("@alloy-ts/buffer-win32-ia32-msvc");
				const bindingPackageVersion = require("@alloy-ts/buffer-win32-ia32-msvc/package.json").version;
				if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
				return binding;
			} catch (e) {
				loadErrors.push(e);
			}
		} else if (process.arch === "arm64") {
			try {
				return require("./my-addon.win32-arm64-msvc.node");
			} catch (e) {
				loadErrors.push(e);
			}
			try {
				const binding = require("@alloy-ts/buffer-win32-arm64-msvc");
				const bindingPackageVersion = require("@alloy-ts/buffer-win32-arm64-msvc/package.json").version;
				if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
				return binding;
			} catch (e) {
				loadErrors.push(e);
			}
		} else loadErrors.push(/* @__PURE__ */ new Error(`Unsupported architecture on Windows: ${process.arch}`));
	} else if (process.platform === "darwin") {
		try {
			return require("./my-addon.darwin-universal.node");
		} catch (e) {
			loadErrors.push(e);
		}
		try {
			const binding = require("@alloy-ts/buffer-darwin-universal");
			const bindingPackageVersion = require("@alloy-ts/buffer-darwin-universal/package.json").version;
			if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
			return binding;
		} catch (e) {
			loadErrors.push(e);
		}
		if (process.arch === "x64") {
			try {
				return require("./my-addon.darwin-x64.node");
			} catch (e) {
				loadErrors.push(e);
			}
			try {
				const binding = require("@alloy-ts/buffer-darwin-x64");
				const bindingPackageVersion = require("@alloy-ts/buffer-darwin-x64/package.json").version;
				if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
				return binding;
			} catch (e) {
				loadErrors.push(e);
			}
		} else if (process.arch === "arm64") {
			try {
				return require("./my-addon.darwin-arm64.node");
			} catch (e) {
				loadErrors.push(e);
			}
			try {
				const binding = require("@alloy-ts/buffer-darwin-arm64");
				const bindingPackageVersion = require("@alloy-ts/buffer-darwin-arm64/package.json").version;
				if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
				return binding;
			} catch (e) {
				loadErrors.push(e);
			}
		} else loadErrors.push(/* @__PURE__ */ new Error(`Unsupported architecture on macOS: ${process.arch}`));
	} else if (process.platform === "freebsd") {
		if (process.arch === "x64") {
			try {
				return require("./my-addon.freebsd-x64.node");
			} catch (e) {
				loadErrors.push(e);
			}
			try {
				const binding = require("@alloy-ts/buffer-freebsd-x64");
				const bindingPackageVersion = require("@alloy-ts/buffer-freebsd-x64/package.json").version;
				if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
				return binding;
			} catch (e) {
				loadErrors.push(e);
			}
		} else if (process.arch === "arm64") {
			try {
				return require("./my-addon.freebsd-arm64.node");
			} catch (e) {
				loadErrors.push(e);
			}
			try {
				const binding = require("@alloy-ts/buffer-freebsd-arm64");
				const bindingPackageVersion = require("@alloy-ts/buffer-freebsd-arm64/package.json").version;
				if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
				return binding;
			} catch (e) {
				loadErrors.push(e);
			}
		} else loadErrors.push(/* @__PURE__ */ new Error(`Unsupported architecture on FreeBSD: ${process.arch}`));
	} else if (process.platform === "linux") {
		if (process.arch === "x64") {
			if (isMusl()) {
				try {
					return require("./my-addon.linux-x64-musl.node");
				} catch (e) {
					loadErrors.push(e);
				}
				try {
					const binding = require("@alloy-ts/buffer-linux-x64-musl");
					const bindingPackageVersion = require("@alloy-ts/buffer-linux-x64-musl/package.json").version;
					if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
					return binding;
				} catch (e) {
					loadErrors.push(e);
				}
			} else {
				try {
					return require("./my-addon.linux-x64-gnu.node");
				} catch (e) {
					loadErrors.push(e);
				}
				try {
					const binding = require("@alloy-ts/buffer-linux-x64-gnu");
					const bindingPackageVersion = require("@alloy-ts/buffer-linux-x64-gnu/package.json").version;
					if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
					return binding;
				} catch (e) {
					loadErrors.push(e);
				}
			}
		} else if (process.arch === "arm64") {
			if (isMusl()) {
				try {
					return require("./my-addon.linux-arm64-musl.node");
				} catch (e) {
					loadErrors.push(e);
				}
				try {
					const binding = require("@alloy-ts/buffer-linux-arm64-musl");
					const bindingPackageVersion = require("@alloy-ts/buffer-linux-arm64-musl/package.json").version;
					if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
					return binding;
				} catch (e) {
					loadErrors.push(e);
				}
			} else {
				try {
					return require("./my-addon.linux-arm64-gnu.node");
				} catch (e) {
					loadErrors.push(e);
				}
				try {
					const binding = require("@alloy-ts/buffer-linux-arm64-gnu");
					const bindingPackageVersion = require("@alloy-ts/buffer-linux-arm64-gnu/package.json").version;
					if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
					return binding;
				} catch (e) {
					loadErrors.push(e);
				}
			}
		} else if (process.arch === "arm") {
			if (isMusl()) {
				try {
					return require("./my-addon.linux-arm-musleabihf.node");
				} catch (e) {
					loadErrors.push(e);
				}
				try {
					const binding = require("@alloy-ts/buffer-linux-arm-musleabihf");
					const bindingPackageVersion = require("@alloy-ts/buffer-linux-arm-musleabihf/package.json").version;
					if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
					return binding;
				} catch (e) {
					loadErrors.push(e);
				}
			} else {
				try {
					return require("./my-addon.linux-arm-gnueabihf.node");
				} catch (e) {
					loadErrors.push(e);
				}
				try {
					const binding = require("@alloy-ts/buffer-linux-arm-gnueabihf");
					const bindingPackageVersion = require("@alloy-ts/buffer-linux-arm-gnueabihf/package.json").version;
					if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
					return binding;
				} catch (e) {
					loadErrors.push(e);
				}
			}
		} else if (process.arch === "loong64") {
			if (isMusl()) {
				try {
					return require("./my-addon.linux-loong64-musl.node");
				} catch (e) {
					loadErrors.push(e);
				}
				try {
					const binding = require("@alloy-ts/buffer-linux-loong64-musl");
					const bindingPackageVersion = require("@alloy-ts/buffer-linux-loong64-musl/package.json").version;
					if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
					return binding;
				} catch (e) {
					loadErrors.push(e);
				}
			} else {
				try {
					return require("./my-addon.linux-loong64-gnu.node");
				} catch (e) {
					loadErrors.push(e);
				}
				try {
					const binding = require("@alloy-ts/buffer-linux-loong64-gnu");
					const bindingPackageVersion = require("@alloy-ts/buffer-linux-loong64-gnu/package.json").version;
					if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
					return binding;
				} catch (e) {
					loadErrors.push(e);
				}
			}
		} else if (process.arch === "riscv64") {
			if (isMusl()) {
				try {
					return require("./my-addon.linux-riscv64-musl.node");
				} catch (e) {
					loadErrors.push(e);
				}
				try {
					const binding = require("@alloy-ts/buffer-linux-riscv64-musl");
					const bindingPackageVersion = require("@alloy-ts/buffer-linux-riscv64-musl/package.json").version;
					if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
					return binding;
				} catch (e) {
					loadErrors.push(e);
				}
			} else {
				try {
					return require("./my-addon.linux-riscv64-gnu.node");
				} catch (e) {
					loadErrors.push(e);
				}
				try {
					const binding = require("@alloy-ts/buffer-linux-riscv64-gnu");
					const bindingPackageVersion = require("@alloy-ts/buffer-linux-riscv64-gnu/package.json").version;
					if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
					return binding;
				} catch (e) {
					loadErrors.push(e);
				}
			}
		} else if (process.arch === "ppc64") {
			try {
				return require("./my-addon.linux-ppc64-gnu.node");
			} catch (e) {
				loadErrors.push(e);
			}
			try {
				const binding = require("@alloy-ts/buffer-linux-ppc64-gnu");
				const bindingPackageVersion = require("@alloy-ts/buffer-linux-ppc64-gnu/package.json").version;
				if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
				return binding;
			} catch (e) {
				loadErrors.push(e);
			}
		} else if (process.arch === "s390x") {
			try {
				return require("./my-addon.linux-s390x-gnu.node");
			} catch (e) {
				loadErrors.push(e);
			}
			try {
				const binding = require("@alloy-ts/buffer-linux-s390x-gnu");
				const bindingPackageVersion = require("@alloy-ts/buffer-linux-s390x-gnu/package.json").version;
				if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
				return binding;
			} catch (e) {
				loadErrors.push(e);
			}
		} else loadErrors.push(/* @__PURE__ */ new Error(`Unsupported architecture on Linux: ${process.arch}`));
	} else if (process.platform === "openharmony") {
		if (process.arch === "arm64") {
			try {
				return require("./my-addon.openharmony-arm64.node");
			} catch (e) {
				loadErrors.push(e);
			}
			try {
				const binding = require("@alloy-ts/buffer-openharmony-arm64");
				const bindingPackageVersion = require("@alloy-ts/buffer-openharmony-arm64/package.json").version;
				if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
				return binding;
			} catch (e) {
				loadErrors.push(e);
			}
		} else if (process.arch === "x64") {
			try {
				return require("./my-addon.openharmony-x64.node");
			} catch (e) {
				loadErrors.push(e);
			}
			try {
				const binding = require("@alloy-ts/buffer-openharmony-x64");
				const bindingPackageVersion = require("@alloy-ts/buffer-openharmony-x64/package.json").version;
				if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
				return binding;
			} catch (e) {
				loadErrors.push(e);
			}
		} else if (process.arch === "arm") {
			try {
				return require("./my-addon.openharmony-arm.node");
			} catch (e) {
				loadErrors.push(e);
			}
			try {
				const binding = require("@alloy-ts/buffer-openharmony-arm");
				const bindingPackageVersion = require("@alloy-ts/buffer-openharmony-arm/package.json").version;
				if (bindingPackageVersion !== "0.0.0" && process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") throw new Error(`Native binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
				return binding;
			} catch (e) {
				loadErrors.push(e);
			}
		} else loadErrors.push(/* @__PURE__ */ new Error(`Unsupported architecture on OpenHarmony: ${process.arch}`));
	} else loadErrors.push(/* @__PURE__ */ new Error(`Unsupported OS: ${process.platform}, architecture: ${process.arch}`));
}
function createLoadErrorChain(errors) {
	return errors.reduce((previous, current) => {
		let message;
		try {
			message = current && typeof current.message === "string" ? current.message : String(current);
		} catch {
			message = "Unknown error";
		}
		const error = new Error(message);
		error.cause = previous;
		return error;
	}, null);
}
const __napiWasiFlavors = ["wasm32-wasi"];
const __napiWasiFlavor = process.env.NAPI_RS_WASI_FLAVOR;
const __napiWasiFlavorRequested = typeof __napiWasiFlavor === "string" && __napiWasiFlavor.length > 0;
if (__napiWasiFlavorRequested && __napiWasiFlavors.indexOf(__napiWasiFlavor) === -1) throw new Error("Unsupported WASI flavor \"" + __napiWasiFlavor + "\". Available flavors: " + __napiWasiFlavors.join(", "));
const forceWasiError = process.env.NAPI_RS_FORCE_WASI === "error";
const forceWasi = process.env.NAPI_RS_FORCE_WASI === "true" || forceWasiError || __napiWasiFlavorRequested;
if (!forceWasi) nativeBinding = requireNative();
if (!nativeBinding || forceWasi) {
	let wasiBinding = null;
	let wasiBindingLoaded = false;
	const wasiBindingErrors = [];
	const __napiWasiResolveCandidate = (specifier, isPackage, localArtifacts) => {
		try {
			require.resolve(specifier);
		} catch (resolveError) {
			if (!resolveError || resolveError.code !== "MODULE_NOT_FOUND") throw resolveError;
			if (isPackage) {
				try {
					require.resolve(specifier + "/package.json");
				} catch (packageError) {
					if (packageError && packageError.code === "MODULE_NOT_FOUND") return resolveError;
					throw resolveError;
				}
				throw resolveError;
			}
			return resolveError;
		}
		if (localArtifacts) {
			let artifactError = null;
			for (let i = 0; i < localArtifacts.length; i++) try {
				require.resolve(localArtifacts[i]);
				return null;
			} catch (resolveError) {
				if (!resolveError || resolveError.code !== "MODULE_NOT_FOUND") throw resolveError;
				artifactError = resolveError;
			}
			return artifactError;
		}
		return null;
	};
	if (!wasiBindingLoaded && (!__napiWasiFlavorRequested || __napiWasiFlavor === "wasm32-wasi")) {
		let candidateError = null;
		let candidateFailed = false;
		try {
			candidateError = __napiWasiResolveCandidate("./my-addon.wasi.cjs", false, ["./my-addon.wasm32-wasi.debug.wasm", "./my-addon.wasm32-wasi.wasm"]);
			candidateFailed = candidateError !== null;
			if (!candidateFailed) {
				wasiBinding = require("./my-addon.wasi.cjs");
				nativeBinding = wasiBinding;
				__napiLoadedBindingTarget = "wasm32-wasi";
				wasiBindingLoaded = true;
			}
		} catch (err) {
			candidateError = err;
			candidateFailed = true;
		}
		if (candidateFailed) {
			wasiBindingErrors.push(candidateError);
			loadErrors.push(candidateError);
		}
	}
	if (!wasiBindingLoaded && (!__napiWasiFlavorRequested || __napiWasiFlavor === "wasm32-wasi")) {
		let candidateError = null;
		let candidateFailed = false;
		try {
			candidateError = __napiWasiResolveCandidate("@alloy-ts/buffer-wasm32-wasi", true, void 0);
			candidateFailed = candidateError !== null;
			if (!candidateFailed) {
				if (process.env.NAPI_RS_ENFORCE_VERSION_CHECK && process.env.NAPI_RS_ENFORCE_VERSION_CHECK !== "0") {
					const bindingPackageVersion = require("@alloy-ts/buffer-wasm32-wasi/package.json").version;
					if (bindingPackageVersion !== "0.0.0") throw new Error(`WASI binding package version mismatch, expected 0.0.0 but got ${bindingPackageVersion}. You can reinstall dependencies to fix this issue.`);
				}
				wasiBinding = require("@alloy-ts/buffer-wasm32-wasi");
				nativeBinding = wasiBinding;
				__napiLoadedBindingTarget = "wasm32-wasi";
				wasiBindingLoaded = true;
			}
		} catch (err) {
			candidateError = err;
			candidateFailed = true;
		}
		if (candidateFailed) {
			wasiBindingErrors.push(candidateError);
			loadErrors.push(candidateError);
		}
	}
	if (!wasiBindingLoaded && forceWasi && !forceWasiError && !__napiWasiFlavorRequested) nativeBinding = requireNative();
	if ((forceWasiError || __napiWasiFlavorRequested) && !wasiBindingLoaded) {
		const error = /* @__PURE__ */ new Error(__napiWasiFlavorRequested ? "WASI binding for flavor \"" + __napiWasiFlavor + "\" not found" : "WASI binding not found and NAPI_RS_FORCE_WASI is set to error");
		error.cause = createLoadErrorChain(wasiBindingErrors);
		throw error;
	}
}
if (!nativeBinding) {
	if (loadErrors.length > 0) {
		const error = /* @__PURE__ */ new Error("Cannot find native binding. npm has a bug related to optional dependencies (https://github.com/npm/cli/issues/4828). Please try `npm i` again after removing both package-lock.json and node_modules directory.");
		error.cause = createLoadErrorChain(loadErrors);
		throw error;
	}
	throw new Error(`Failed to load native binding`);
}
const { Blob: Blob$1, Bytes: Bytes$1, File: File$1, add, atob: atob$1, btoa: btoa$1, bufByteLength, bufCompare, bufConcat, bufCopyBytesFrom, bufDecode, bufEncode, bufFill, bufIncludes, bufIndexOf, bufIsEncoding, bufLastIndexOf, bufStringLength, bufWrite, isAscii: isAscii$1, isLatin1, isUtf8: isUtf8$1, resolveObjectURL: resolveObjectURL$1, transcode: transcode$1 } = nativeBinding;
const __napiBindingTarget = __napiLoadedBindingTarget;
//#endregion
//#region src/buffer.ts
const native = app_exports;
var Buffer$1 = class Buffer$1 extends Uint8Array {
	static poolSize = native.poolSize ?? 65536;
	static isBuffer(obj) {
		return obj instanceof Buffer$1 || obj != null && typeof obj === "object" && obj._isBuffer === true;
	}
	static isEncoding(encoding) {
		if (typeof encoding !== "string" || encoding === "") return false;
		return native.bufIsEncoding(encoding);
	}
	static alloc(size, fill, encoding) {
		if (typeof size !== "number" || Number.isNaN(size)) throw new TypeError("The \"size\" argument must be of type number");
		if (size < 0 || size > (native.kMaxLength ?? 2147483647)) throw new RangeError("The value of \"size\" is out of range");
		const buf = Buffer$1.allocUnsafe(size);
		if (fill !== void 0) buf.fill(fill, 0, size, encoding);
		else buf.fill(0);
		return buf;
	}
	static allocUnsafe(size) {
		if (typeof size !== "number" || Number.isNaN(size)) throw new TypeError("The \"size\" argument must be of type number");
		if (size < 0 || size > (native.kMaxLength ?? 2147483647)) throw new RangeError("The value of \"size\" is out of range");
		return new Buffer$1(size);
	}
	static allocUnsafeSlow(size) {
		return Buffer$1.allocUnsafe(size);
	}
	static byteLength(string, encoding) {
		if (typeof string !== "string") {
			if (ArrayBuffer.isView(string)) return string.byteLength;
			if (string instanceof ArrayBuffer || typeof SharedArrayBuffer !== "undefined" && string instanceof SharedArrayBuffer) return string.byteLength;
			throw new TypeError("The \"string\" argument must be of type string or an instance of Buffer or ArrayBuffer");
		}
		return native.bufByteLength(string, encoding);
	}
	static compare(buf1, buf2) {
		if (!(buf1 instanceof Uint8Array) || !(buf2 instanceof Uint8Array)) throw new TypeError("The \"buf1\" and \"buf2\" arguments must be an instance of Uint8Array");
		const res = native.bufCompare(buf1, buf2);
		return res < 0 ? -1 : res > 0 ? 1 : 0;
	}
	static concat(list, totalLength) {
		if (!Array.isArray(list)) throw new TypeError("The \"list\" argument must be an instance of Array");
		if (list.length === 0 || totalLength === 0) return Buffer$1.alloc(0);
		if (totalLength === void 0) {
			totalLength = 0;
			for (const b of list) totalLength += b.length;
		}
		const bufs = list.map((item) => item instanceof Uint8Array ? item : new Uint8Array(item));
		const res = native.bufConcat(bufs, totalLength);
		return Buffer$1.from(res.buffer, res.byteOffset, res.byteLength);
	}
	static copyBytesFrom(view, offset, length) {
		if (!ArrayBuffer.isView(view)) throw new TypeError("The \"view\" argument must be an instance of ArrayBufferView");
		const res = native.bufCopyBytesFrom(view, offset, length);
		return Buffer$1.from(res.buffer, res.byteOffset, res.byteLength);
	}
	static from(value, encodingOrOffset, length) {
		if (typeof value === "string") {
			const res = native.bufDecode(value, encodingOrOffset);
			return Buffer$1.from(res.buffer, res.byteOffset, res.byteLength);
		}
		if (value instanceof ArrayBuffer || typeof SharedArrayBuffer !== "undefined" && value instanceof SharedArrayBuffer) {
			const byteOffset = encodingOrOffset ?? 0;
			const len = length ?? value.byteLength - byteOffset;
			return new Buffer$1(value, byteOffset, len);
		}
		if (ArrayBuffer.isView(value)) {
			const u8 = new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
			const res = new Buffer$1(u8.length);
			res.set(u8);
			return res;
		}
		if (Array.isArray(value) || value != null && typeof value === "object" && typeof value.length === "number") {
			const arr = value;
			const res = new Buffer$1(arr.length);
			for (let i = 0; i < arr.length; i++) res[i] = Number(arr[i]) & 255 || 0;
			return res;
		}
		if (value != null && typeof value === "object") {
			if (typeof value.valueOf === "function" && value.valueOf() !== value) return Buffer$1.from(value.valueOf(), encodingOrOffset, length);
			if (typeof value[Symbol.toPrimitive] === "function") return Buffer$1.from(value[Symbol.toPrimitive]("string"), encodingOrOffset, length);
		}
		throw new TypeError("First argument must be a string, Buffer, ArrayBuffer, Array, or Array-like object.");
	}
	static of(...items) {
		return Buffer$1.from(items);
	}
	get _isBuffer() {
		return true;
	}
	_dv() {
		return new DataView(this.buffer, this.byteOffset, this.byteLength);
	}
	toString(encoding, start, end) {
		return native.bufEncode(this, encoding, start, end);
	}
	write(string, offsetOrEncoding, lengthOrEncoding, encoding) {
		let offset = 0;
		let length = this.length;
		let enc = encoding ?? "utf8";
		if (typeof offsetOrEncoding === "string") enc = offsetOrEncoding;
		else if (typeof offsetOrEncoding === "number") {
			offset = offsetOrEncoding;
			if (typeof lengthOrEncoding === "string") enc = lengthOrEncoding;
			else if (typeof lengthOrEncoding === "number") length = lengthOrEncoding;
		}
		return native.bufWrite(this, string, offset, length, enc);
	}
	toJSON() {
		return {
			type: "Buffer",
			data: Array.from(this)
		};
	}
	equals(otherBuffer) {
		if (!(otherBuffer instanceof Uint8Array)) throw new TypeError("Argument must be a Uint8Array or Buffer");
		return Buffer$1.compare(this, otherBuffer) === 0;
	}
	compare(target, targetStart, targetEnd, sourceStart, sourceEnd) {
		if (!(target instanceof Uint8Array)) throw new TypeError("Argument must be a Uint8Array or Buffer");
		const tStart = targetStart ?? 0;
		const tEnd = targetEnd ?? target.byteLength;
		const sStart = sourceStart ?? 0;
		const sEnd = sourceEnd ?? this.byteLength;
		if (tStart < 0 || sStart < 0 || tEnd > target.byteLength || sEnd > this.byteLength) throw new RangeError("ERR_OUT_OF_RANGE: Index out of range");
		const srcSlice = this.subarray(sStart, sEnd);
		const tgtSlice = target.subarray(tStart, tEnd);
		return Buffer$1.compare(srcSlice, tgtSlice);
	}
	copy(target, targetStart, sourceStart, sourceEnd) {
		if (!(target instanceof Uint8Array)) throw new TypeError("Argument must be a Uint8Array or Buffer");
		let tStart = targetStart ?? 0;
		let sStart = sourceStart ?? 0;
		let sEnd = sourceEnd ?? this.length;
		if (tStart < 0) tStart = 0;
		if (sStart < 0) sStart = 0;
		if (sEnd > this.length) sEnd = this.length;
		if (sStart >= sEnd || tStart >= target.length) return 0;
		const toCopy = Math.min(sEnd - sStart, target.length - tStart);
		target.set(this.subarray(sStart, sStart + toCopy), tStart);
		return toCopy;
	}
	fill(value, offset, end, encoding) {
		native.bufFill(this, value, offset, end, encoding);
		return this;
	}
	indexOf(value, byteOffset, encoding) {
		return native.bufIndexOf(this, value, byteOffset, void 0, encoding);
	}
	lastIndexOf(value, byteOffset, encoding) {
		return native.bufLastIndexOf(this, value, byteOffset, void 0, encoding);
	}
	includes(value, byteOffset, encoding) {
		return native.bufIncludes(this, value, byteOffset, void 0, encoding);
	}
	slice(start, end) {
		return this.subarray(start, end);
	}
	subarray(start, end) {
		const sub = super.subarray(start, end);
		return Buffer$1.from(sub.buffer, sub.byteOffset, sub.byteLength);
	}
	reverse() {
		super.reverse();
		return this;
	}
	swap16() {
		const len = this.length;
		if (len % 2 !== 0) throw new RangeError("ERR_INVALID_BUFFER_SIZE: Buffer size must be a multiple of 16-bits");
		for (let i = 0; i < len; i += 2) {
			const val = this[i];
			this[i] = this[i + 1];
			this[i + 1] = val;
		}
		return this;
	}
	swap32() {
		const len = this.length;
		if (len % 4 !== 0) throw new RangeError("ERR_INVALID_BUFFER_SIZE: Buffer size must be a multiple of 32-bits");
		for (let i = 0; i < len; i += 4) {
			const b0 = this[i];
			const b1 = this[i + 1];
			const b2 = this[i + 2];
			const b3 = this[i + 3];
			this[i] = b3;
			this[i + 1] = b2;
			this[i + 2] = b1;
			this[i + 3] = b0;
		}
		return this;
	}
	swap64() {
		const len = this.length;
		if (len % 8 !== 0) throw new RangeError("ERR_INVALID_BUFFER_SIZE: Buffer size must be a multiple of 64-bits");
		for (let i = 0; i < len; i += 8) {
			const b0 = this[i];
			const b1 = this[i + 1];
			const b2 = this[i + 2];
			const b3 = this[i + 3];
			const b4 = this[i + 4];
			const b5 = this[i + 5];
			const b6 = this[i + 6];
			const b7 = this[i + 7];
			this[i] = b7;
			this[i + 1] = b6;
			this[i + 2] = b5;
			this[i + 3] = b4;
			this[i + 4] = b3;
			this[i + 5] = b2;
			this[i + 6] = b1;
			this[i + 7] = b0;
		}
		return this;
	}
	readInt8(offset = 0) {
		return this._dv().getInt8(offset);
	}
	readUInt8(offset = 0) {
		return this._dv().getUint8(offset);
	}
	readUint8(offset = 0) {
		return this.readUInt8(offset);
	}
	readInt16LE(offset = 0) {
		return this._dv().getInt16(offset, true);
	}
	readInt16BE(offset = 0) {
		return this._dv().getInt16(offset, false);
	}
	readUInt16LE(offset = 0) {
		return this._dv().getUint16(offset, true);
	}
	readUint16LE(offset = 0) {
		return this.readUInt16LE(offset);
	}
	readUInt16BE(offset = 0) {
		return this._dv().getUint16(offset, false);
	}
	readUint16BE(offset = 0) {
		return this.readUInt16BE(offset);
	}
	readInt32LE(offset = 0) {
		return this._dv().getInt32(offset, true);
	}
	readInt32BE(offset = 0) {
		return this._dv().getInt32(offset, false);
	}
	readUInt32LE(offset = 0) {
		return this._dv().getUint32(offset, true);
	}
	readUint32LE(offset = 0) {
		return this.readUInt32LE(offset);
	}
	readUInt32BE(offset = 0) {
		return this._dv().getUint32(offset, false);
	}
	readUint32BE(offset = 0) {
		return this.readUInt32BE(offset);
	}
	readFloatLE(offset = 0) {
		return this._dv().getFloat32(offset, true);
	}
	readFloatBE(offset = 0) {
		return this._dv().getFloat32(offset, false);
	}
	readDoubleLE(offset = 0) {
		return this._dv().getFloat64(offset, true);
	}
	readDoubleBE(offset = 0) {
		return this._dv().getFloat64(offset, false);
	}
	readBigInt64LE(offset = 0) {
		return this._dv().getBigInt64(offset, true);
	}
	readBigInt64BE(offset = 0) {
		return this._dv().getBigInt64(offset, false);
	}
	readBigUInt64LE(offset = 0) {
		return this._dv().getBigUint64(offset, true);
	}
	readBigUint64LE(offset = 0) {
		return this.readBigUInt64LE(offset);
	}
	readBigUInt64BE(offset = 0) {
		return this._dv().getBigUint64(offset, false);
	}
	readBigUint64BE(offset = 0) {
		return this.readBigUInt64BE(offset);
	}
	readUIntLE(offset, byteLength) {
		let val = this[offset];
		let mul = 1;
		let i = 0;
		while (++i < byteLength && (mul *= 256)) val += this[offset + i] * mul;
		return val;
	}
	readUintLE(offset, byteLength) {
		return this.readUIntLE(offset, byteLength);
	}
	readUIntBE(offset, byteLength) {
		let len = byteLength;
		let val = this[offset + --len];
		let mul = 1;
		while (len > 0 && (mul *= 256)) val += this[offset + --len] * mul;
		return val;
	}
	readUintBE(offset, byteLength) {
		return this.readUIntBE(offset, byteLength);
	}
	readIntLE(offset, byteLength) {
		let val = this.readUIntLE(offset, byteLength);
		const limit = 2 ** (8 * byteLength - 1);
		if (val >= limit) val -= limit * 2;
		return val;
	}
	readIntBE(offset, byteLength) {
		let val = this.readUIntBE(offset, byteLength);
		const limit = 2 ** (8 * byteLength - 1);
		if (val >= limit) val -= limit * 2;
		return val;
	}
	writeInt8(value, offset = 0) {
		this._dv().setInt8(offset, value);
		return offset + 1;
	}
	writeUInt8(value, offset = 0) {
		this._dv().setUint8(offset, value);
		return offset + 1;
	}
	writeUint8(value, offset = 0) {
		return this.writeUInt8(value, offset);
	}
	writeInt16LE(value, offset = 0) {
		this._dv().setInt16(offset, value, true);
		return offset + 2;
	}
	writeInt16BE(value, offset = 0) {
		this._dv().setInt16(offset, value, false);
		return offset + 2;
	}
	writeUInt16LE(value, offset = 0) {
		this._dv().setUint16(offset, value, true);
		return offset + 2;
	}
	writeUint16LE(value, offset = 0) {
		return this.writeUInt16LE(value, offset);
	}
	writeUInt16BE(value, offset = 0) {
		this._dv().setUint16(offset, value, false);
		return offset + 2;
	}
	writeUint16BE(value, offset = 0) {
		return this.writeUInt16BE(value, offset);
	}
	writeInt32LE(value, offset = 0) {
		this._dv().setInt32(offset, value, true);
		return offset + 4;
	}
	writeInt32BE(value, offset = 0) {
		this._dv().setInt32(offset, value, false);
		return offset + 4;
	}
	writeUInt32LE(value, offset = 0) {
		this._dv().setUint32(offset, value, true);
		return offset + 4;
	}
	writeUint32LE(value, offset = 0) {
		return this.writeUInt32LE(value, offset);
	}
	writeUInt32BE(value, offset = 0) {
		this._dv().setUint32(offset, value, false);
		return offset + 4;
	}
	writeUint32BE(value, offset = 0) {
		return this.writeUInt32BE(value, offset);
	}
	writeFloatLE(value, offset = 0) {
		this._dv().setFloat32(offset, value, true);
		return offset + 4;
	}
	writeFloatBE(value, offset = 0) {
		this._dv().setFloat32(offset, value, false);
		return offset + 4;
	}
	writeDoubleLE(value, offset = 0) {
		this._dv().setFloat64(offset, value, true);
		return offset + 8;
	}
	writeDoubleBE(value, offset = 0) {
		this._dv().setFloat64(offset, value, false);
		return offset + 8;
	}
	writeBigInt64LE(value, offset = 0) {
		this._dv().setBigInt64(offset, value, true);
		return offset + 8;
	}
	writeBigInt64BE(value, offset = 0) {
		this._dv().setBigInt64(offset, value, false);
		return offset + 8;
	}
	writeBigUInt64LE(value, offset = 0) {
		this._dv().setBigUint64(offset, value, true);
		return offset + 8;
	}
	writeBigUint64LE(value, offset = 0) {
		return this.writeBigUInt64LE(value, offset);
	}
	writeBigUInt64BE(value, offset = 0) {
		this._dv().setBigUint64(offset, value, false);
		return offset + 8;
	}
	writeBigUint64BE(value, offset = 0) {
		return this.writeBigUInt64BE(value, offset);
	}
	writeUIntLE(value, offset, byteLength) {
		let i = 0;
		let mul = 1;
		this[offset] = value & 255;
		while (++i < byteLength && (mul *= 256)) this[offset + i] = value / mul & 255;
		return offset + byteLength;
	}
	writeUintLE(value, offset, byteLength) {
		return this.writeUIntLE(value, offset, byteLength);
	}
	writeUIntBE(value, offset, byteLength) {
		let i = byteLength - 1;
		let mul = 1;
		this[offset + i] = value & 255;
		while (--i >= 0 && (mul *= 256)) this[offset + i] = value / mul & 255;
		return offset + byteLength;
	}
	writeUintBE(value, offset, byteLength) {
		return this.writeUIntBE(value, offset, byteLength);
	}
	writeIntLE(value, offset, byteLength) {
		if (value < 0) value = 2 ** (8 * byteLength) + value;
		return this.writeUIntLE(value, offset, byteLength);
	}
	writeIntBE(value, offset, byteLength) {
		if (value < 0) value = 2 ** (8 * byteLength) + value;
		return this.writeUIntBE(value, offset, byteLength);
	}
};
const Blob = native.Blob;
const Bytes = native.Bytes;
const File = native.File;
const atob = native.atob;
const btoa = native.btoa;
const isAscii = native.isAscii;
const isUtf8 = native.isUtf8;
const resolveObjectURL = native.resolveObjectURL;
function transcode(source, fromEncoding, toEncoding) {
	const res = native.transcode(source, fromEncoding, toEncoding);
	return Buffer$1.from(res.buffer, res.byteOffset, res.byteLength);
}
const constants = {
	MAX_LENGTH: native.kMaxLength ?? 2147483647,
	MAX_STRING_LENGTH: native.kStringMaxLength ?? 536870888
};
const INSPECT_MAX_BYTES = native.INSPECT_MAX_BYTES ?? 50;
const kMaxLength = native.kMaxLength ?? 2147483647;
const kStringMaxLength = native.kStringMaxLength ?? 536870888;
function SlowBuffer(size) {
	return Buffer$1.allocUnsafe(size);
}
SlowBuffer.prototype = Buffer$1.prototype;
//#endregion
//#region src/main.ts
const main = () => {
	return "Hello, world!";
};
//#endregion
export { Blob, Buffer$1 as Buffer, Bytes, File, INSPECT_MAX_BYTES, SlowBuffer, atob, btoa, constants, isAscii, isUtf8, kMaxLength, kStringMaxLength, main, resolveObjectURL, transcode };
