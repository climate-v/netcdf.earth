import Module from './visualize';

const VALUE_TYPES = [
    'i8',
    'i16',
    'i32',
    'u8',
    'u16',
    'u32',
    'f32',
    'f64'
];

function setupAllocationConfig(moduleInstance) {
    return {
        'i32': {
            heap: moduleInstance.HEAP32,
            array: Int32Array
        },
        'i16': {
            heap: moduleInstance.HEAP16,
            array: Int16Array
        },
        'i8': {
            heap: moduleInstance.HEAP8,
            array: Int8Array
        },
        'u32': {
            heap: moduleInstance.HEAPU32,
            array: Uint32Array
        },
        'u16': {
            heap: moduleInstance.HEAPU16,
            array: Uint16Array
        },
        'u8': {
            heap: moduleInstance.HEAPU8,
            array: Uint8Array
        },
        'f32': {
            heap: moduleInstance.HEAPF32,
            array: Float32Array
        },
        'f64': {
            heap: moduleInstance.HEAPF64,
            array: Float64Array
        }
    }
}

function setupAllocArray(module, allocationConfiguration) {
    return (type, array) => {
        let conf = allocationConfiguration[type];
        if(conf == null) {
            throw new Error("Could not find allocation configuration for type: " + type);
        }

        const arrayCreator = conf.array;
        const size = array.length;
        const byteSize = arrayCreator.BYTES_PER_ELEMENT;
        const offset = module._malloc(size * byteSize);
        const alignedOffset = offset / byteSize;
        conf.heap.set(new arrayCreator(array), alignedOffset);
        return {
            data: conf.heap.subarray(alignedOffset, alignedOffset + size),
            offset
        };
    };
}

function setupCwrapArrayParams(module, allocArray) {
    return (callName, returnType, argTypes, arrayTypes) => {
        let arrayIndices = [];
        let targetArgTypes = [];
        for(let i = argTypes.length - 1; i >= 0; i--) {
            const currentType = argTypes[i];
            if(currentType === 'array') {
                arrayIndices = arrayIndices.map(index => index + 1);
                arrayIndices.unshift(i);
                targetArgTypes.unshift('number', 'number');
            } else {
                targetArgTypes.unshift(currentType);
            }
        }

        const typeOfArrayNr = (number) => {
            if(arrayTypes == null || arrayTypes.length === 0 || arrayTypes.length <= number) {
                return 'i32';
            } else {
                return arrayTypes[number];
            }
        }

        const wrap = module.cwrap(callName, returnType, targetArgTypes);

        return (...args) => {
            const newArgs = [...args];
            let arrayCount = 0;
            const buffers = [];
            for(let index of arrayIndices) {
                const arr = newArgs[index];
                const type = typeOfArrayNr(arrayCount);
                const allocation = allocArray(type, arr);
                newArgs[index] = allocation.offset;
                buffers.push(allocation.offset);
                newArgs.splice(index + 1, 0, arr.length);
                arrayCount++;
            }

            let result;
            try {
                result = wrap(...newArgs);
            } finally {
                for(let buffer of buffers) { // is this necessary? rust should free it on deref
                    module._free(buffer);
                }
            }

            return result;
        }
    };
}

function setupCwrapArrayReturn(module, allocationConfig) {
    return (api, cwrap, type) => {
        const conf = allocationConfig[type];
        if(conf == null) {
            throw new Error("Could not find allocation configuration for type: " + type);
        }

        const usedHeap = conf.heap;
        const arrayCreator = conf.array;
        return (...args) => {
            const result = cwrap(...args);
            const arrayPointer = module.HEAPU32[result / Uint32Array.BYTES_PER_ELEMENT];
            const arrayLen = module.HEAPU32[result / Uint32Array.BYTES_PER_ELEMENT + 1];
            const array = new arrayCreator(usedHeap.buffer, arrayPointer, arrayLen);
            const newReturn = [...array];
            api.dropBytes(result);
            return newReturn;
        };
    };
}

function constructApi(moduleInstance) {
    const ALLOCATION_CONFIG = setupAllocationConfig(moduleInstance);
    const allocArray = setupAllocArray(moduleInstance, ALLOCATION_CONFIG);
    const cwrapArrayParams = setupCwrapArrayParams(moduleInstance, allocArray);
    const cwrapArrayReturn = setupCwrapArrayReturn(moduleInstance, ALLOCATION_CONFIG);

    const api = {
        FS: moduleInstance.FS,
        createNetcdfFile(data) {
            return new NetCDFFile(this, data);
        },
        openFile: moduleInstance.cwrap('open_file', 'boolean', ['string']),
        getStringAttribute: moduleInstance.cwrap('get_string_attribute', 'string', ['string']),
        getVariables: moduleInstance.cwrap('get_variables', 'string', []),
        getDimensions: moduleInstance.cwrap('get_dimensions', 'string', []),
        closeFile: moduleInstance.cwrap('close_file', null, []),
        getVariableDimensions: moduleInstance.cwrap('get_variable_dimensions', 'string', ['string']),
        dropBytes: moduleInstance.cwrap('drop_bytes', null, ['number']),
        getVariableType: moduleInstance.cwrap('get_variable_type', 'string', ['string']),
        getVariableStringAttribute: moduleInstance.cwrap('get_variable_string_attribute', 'string', ['string', 'string']),
        getDimensionLength: moduleInstance.cwrap('get_dimension_len', 'number', ['string'])
    }

    api.getTitle = () => api.getStringAttribute('title');
    api.getVariableValue = (variable, ...args) => {
        const variableType = api.getVariableType(variable);
        return api[`getVariableValue${variableType.toUpperCase()}`](variable, ...args);
    };
    api.getVariableValues = (variable, ...args) => {
        const variableType = api.getVariableType(variable);
        return api[`getVariableValues${variableType.toUpperCase()}`](variable, ...args);
    };

    api.getAllVariableValues = (variable) => {
        const variableType = api.getVariableType(variable);
        return api[`getAllVariableValues${variableType.toUpperCase()}`](variable);
    };

    for(let varType of VALUE_TYPES) {
        api[`getVariableValues${varType.toUpperCase()}`] = cwrapArrayReturn(api, cwrapArrayParams(`get_${varType}_values_for`, 'number', ['string', 'array', 'array'], ['i32', 'i32']), varType);
        api[`getAllVariableValues${varType.toUpperCase()}`] = (variable) => api[`getVariableValues${varType.toUpperCase()}`](variable, [], 0, [], 0);
        api[`getVariableValue${varType.toUpperCase()}`] = cwrapArrayParams(`get_${varType}_value_for`, 'number', ['string', 'array'], ['i32']);
    }
    return api;
}

class NetCDFFile {
    constructor(api, { type, ref, name = "" }) {
        this.api = api;
        this.FS = api.FS;
        this.fileRef = ref;
        this.fileType = type;
        this.mountedPath = null;
        this.isOpen = false;
        if(type === 'file') {
            this.name = ref.name;
        } else {
            this.name = name;
        }
    }

    async _mount() {
        const path = `/${this.name}`;
        let buffer;
        if(this.fileType === 'buffer') {
            buffer = this.fileRef;
        } else {
            buffer = await this.fileRef.arrayBuffer();
        }
        const fp = this.FS.open(path, 'w+');
        const view = new Uint8Array(buffer);
        this.FS.write(fp, view, 0, view.length, 0);
        this.FS.close(fp);
        this.mountedPath = path;
    }

    _isMounted() {
        return this.mountedPath != null;
    }

    async open() {
        if(!this._isMounted()) {
            await this._mount();
        }

        this.isOpen = this.api.openFile(this.mountedPath);
        return this.isOpen;
    }

    _unmount() {
        this.FS.unlink(this.mountedPath);
        this.mountedPath = null;
    }

    close() {
        if(this.isOpen) {
            this.api.closeFile();
        }

        if(this.mountedPath != null) {
            this._unmount();
        }
    }
}

export function createApi() {
    return Module({
        locateFile(path) {
            return "/" + path;
        }
    }).then(module => {
        return constructApi(module);
    });
}
