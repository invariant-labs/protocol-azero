export const abi = `{
    "source": {
      "hash": "0xcb2fa35959c8603817b53a15b8d56b964ee3eafb07daf1f8dc1df63d01029e00",
      "language": "ink! 4.3.0",
      "compiler": "rustc 1.75.0-nightly",
      "build_info": {
        "build_mode": "Release",
        "cargo_contract_version": "3.2.0",
        "rust_toolchain": "nightly-x86_64-unknown-linux-gnu",
        "wasm_opt_settings": {
          "keep_debug_symbols": false,
          "optimization_passes": "Z"
        }
      }
    },
    "contract": {
      "name": "psp22",
      "version": "0.2.1",
      "authors": [
        "Cardinal"
      ],
      "description": "Minimal implementation of PSP22 token standard in pure ink!",
      "repository": "https://github.com/Cardinal-Cryptography/PSP22",
      "homepage": "https://github.com/Cardinal-Cryptography/PSP22",
      "license": "Apache-2.0"
    },
    "spec": {
      "constructors": [
        {
          "args": [
            {
              "label": "supply",
              "type": {
                "displayName": [
                  "u128"
                ],
                "type": 0
              }
            },
            {
              "label": "name",
              "type": {
                "displayName": [
                  "Option"
                ],
                "type": 3
              }
            },
            {
              "label": "symbol",
              "type": {
                "displayName": [
                  "Option"
                ],
                "type": 3
              }
            },
            {
              "label": "decimals",
              "type": {
                "displayName": [
                  "u8"
                ],
                "type": 2
              }
            }
          ],
          "default": false,
          "docs": [],
          "label": "new",
          "payable": false,
          "returnType": {
            "displayName": [
              "ink_primitives",
              "ConstructorResult"
            ],
            "type": 4
          },
          "selector": "0x9bae9d5e"
        }
      ],
      "docs": [],
      "environment": {
        "accountId": {
          "displayName": [
            "AccountId"
          ],
          "type": 8
        },
        "balance": {
          "displayName": [
            "Balance"
          ],
          "type": 0
        },
        "blockNumber": {
          "displayName": [
            "BlockNumber"
          ],
          "type": 19
        },
        "chainExtension": {
          "displayName": [
            "ChainExtension"
          ],
          "type": 20
        },
        "hash": {
          "displayName": [
            "Hash"
          ],
          "type": 17
        },
        "maxEventTopics": 4,
        "timestamp": {
          "displayName": [
            "Timestamp"
          ],
          "type": 18
        }
      },
      "events": [
        {
          "args": [
            {
              "docs": [],
              "indexed": true,
              "label": "owner",
              "type": {
                "displayName": [
                  "AccountId"
                ],
                "type": 8
              }
            },
            {
              "docs": [],
              "indexed": true,
              "label": "spender",
              "type": {
                "displayName": [
                  "AccountId"
                ],
                "type": 8
              }
            },
            {
              "docs": [],
              "indexed": false,
              "label": "amount",
              "type": {
                "displayName": [
                  "u128"
                ],
                "type": 0
              }
            }
          ],
          "docs": [],
          "label": "Approval"
        },
        {
          "args": [
            {
              "docs": [],
              "indexed": true,
              "label": "from",
              "type": {
                "displayName": [
                  "Option"
                ],
                "type": 16
              }
            },
            {
              "docs": [],
              "indexed": true,
              "label": "to",
              "type": {
                "displayName": [
                  "Option"
                ],
                "type": 16
              }
            },
            {
              "docs": [],
              "indexed": false,
              "label": "value",
              "type": {
                "displayName": [
                  "u128"
                ],
                "type": 0
              }
            }
          ],
          "docs": [],
          "label": "Transfer"
        }
      ],
      "lang_error": {
        "displayName": [
          "ink",
          "LangError"
        ],
        "type": 6
      },
      "messages": [
        {
          "args": [],
          "default": false,
          "docs": [],
          "label": "PSP22::total_supply",
          "mutates": false,
          "payable": false,
          "returnType": {
            "displayName": [
              "ink",
              "MessageResult"
            ],
            "type": 7
          },
          "selector": "0x162df8c2"
        },
        {
          "args": [
            {
              "label": "owner",
              "type": {
                "displayName": [
                  "AccountId"
                ],
                "type": 8
              }
            }
          ],
          "default": false,
          "docs": [],
          "label": "PSP22::balance_of",
          "mutates": false,
          "payable": false,
          "returnType": {
            "displayName": [
              "ink",
              "MessageResult"
            ],
            "type": 7
          },
          "selector": "0x6568382f"
        },
        {
          "args": [
            {
              "label": "owner",
              "type": {
                "displayName": [
                  "AccountId"
                ],
                "type": 8
              }
            },
            {
              "label": "spender",
              "type": {
                "displayName": [
                  "AccountId"
                ],
                "type": 8
              }
            }
          ],
          "default": false,
          "docs": [],
          "label": "PSP22::allowance",
          "mutates": false,
          "payable": false,
          "returnType": {
            "displayName": [
              "ink",
              "MessageResult"
            ],
            "type": 7
          },
          "selector": "0x4d47d921"
        },
        {
          "args": [
            {
              "label": "to",
              "type": {
                "displayName": [
                  "AccountId"
                ],
                "type": 8
              }
            },
            {
              "label": "value",
              "type": {
                "displayName": [
                  "u128"
                ],
                "type": 0
              }
            },
            {
              "label": "_data",
              "type": {
                "displayName": [
                  "Vec"
                ],
                "type": 10
              }
            }
          ],
          "default": false,
          "docs": [],
          "label": "PSP22::transfer",
          "mutates": true,
          "payable": false,
          "returnType": {
            "displayName": [
              "ink",
              "MessageResult"
            ],
            "type": 11
          },
          "selector": "0xdb20f9f5"
        },
        {
          "args": [
            {
              "label": "from",
              "type": {
                "displayName": [
                  "AccountId"
                ],
                "type": 8
              }
            },
            {
              "label": "to",
              "type": {
                "displayName": [
                  "AccountId"
                ],
                "type": 8
              }
            },
            {
              "label": "value",
              "type": {
                "displayName": [
                  "u128"
                ],
                "type": 0
              }
            },
            {
              "label": "_data",
              "type": {
                "displayName": [
                  "Vec"
                ],
                "type": 10
              }
            }
          ],
          "default": false,
          "docs": [],
          "label": "PSP22::transfer_from",
          "mutates": true,
          "payable": false,
          "returnType": {
            "displayName": [
              "ink",
              "MessageResult"
            ],
            "type": 11
          },
          "selector": "0x54b3c76e"
        },
        {
          "args": [
            {
              "label": "spender",
              "type": {
                "displayName": [
                  "AccountId"
                ],
                "type": 8
              }
            },
            {
              "label": "value",
              "type": {
                "displayName": [
                  "u128"
                ],
                "type": 0
              }
            }
          ],
          "default": false,
          "docs": [],
          "label": "PSP22::approve",
          "mutates": true,
          "payable": false,
          "returnType": {
            "displayName": [
              "ink",
              "MessageResult"
            ],
            "type": 11
          },
          "selector": "0xb20f1bbd"
        },
        {
          "args": [
            {
              "label": "spender",
              "type": {
                "displayName": [
                  "AccountId"
                ],
                "type": 8
              }
            },
            {
              "label": "delta_value",
              "type": {
                "displayName": [
                  "u128"
                ],
                "type": 0
              }
            }
          ],
          "default": false,
          "docs": [],
          "label": "PSP22::increase_allowance",
          "mutates": true,
          "payable": false,
          "returnType": {
            "displayName": [
              "ink",
              "MessageResult"
            ],
            "type": 11
          },
          "selector": "0x96d6b57a"
        },
        {
          "args": [
            {
              "label": "spender",
              "type": {
                "displayName": [
                  "AccountId"
                ],
                "type": 8
              }
            },
            {
              "label": "delta_value",
              "type": {
                "displayName": [
                  "u128"
                ],
                "type": 0
              }
            }
          ],
          "default": false,
          "docs": [],
          "label": "PSP22::decrease_allowance",
          "mutates": true,
          "payable": false,
          "returnType": {
            "displayName": [
              "ink",
              "MessageResult"
            ],
            "type": 11
          },
          "selector": "0xfecb57d5"
        },
        {
          "args": [],
          "default": false,
          "docs": [],
          "label": "PSP22Metadata::token_name",
          "mutates": false,
          "payable": false,
          "returnType": {
            "displayName": [
              "ink",
              "MessageResult"
            ],
            "type": 14
          },
          "selector": "0x3d261bd4"
        },
        {
          "args": [],
          "default": false,
          "docs": [],
          "label": "PSP22Metadata::token_symbol",
          "mutates": false,
          "payable": false,
          "returnType": {
            "displayName": [
              "ink",
              "MessageResult"
            ],
            "type": 14
          },
          "selector": "0x34205be5"
        },
        {
          "args": [],
          "default": false,
          "docs": [],
          "label": "PSP22Metadata::token_decimals",
          "mutates": false,
          "payable": false,
          "returnType": {
            "displayName": [
              "ink",
              "MessageResult"
            ],
            "type": 15
          },
          "selector": "0x7271b782"
        },
        {
          "args": [
            {
              "label": "value",
              "type": {
                "displayName": [
                  "u128"
                ],
                "type": 0
              }
            }
          ],
          "default": false,
          "docs": [],
          "label": "PSP22Mintable::mint",
          "mutates": true,
          "payable": false,
          "returnType": {
            "displayName": [
              "ink",
              "MessageResult"
            ],
            "type": 11
          },
          "selector": "0xfc3c75d4"
        },
        {
          "args": [
            {
              "label": "value",
              "type": {
                "displayName": [
                  "u128"
                ],
                "type": 0
              }
            }
          ],
          "default": false,
          "docs": [],
          "label": "PSP22Burnable::burn",
          "mutates": true,
          "payable": false,
          "returnType": {
            "displayName": [
              "ink",
              "MessageResult"
            ],
            "type": 11
          },
          "selector": "0x7a9da510"
        }
      ]
    },
    "storage": {
      "root": {
        "layout": {
          "struct": {
            "fields": [
              {
                "layout": {
                  "struct": {
                    "fields": [
                      {
                        "layout": {
                          "leaf": {
                            "key": "0x00000000",
                            "ty": 0
                          }
                        },
                        "name": "total_supply"
                      },
                      {
                        "layout": {
                          "root": {
                            "layout": {
                              "leaf": {
                                "key": "0x45c746d4",
                                "ty": 0
                              }
                            },
                            "root_key": "0x45c746d4"
                          }
                        },
                        "name": "balances"
                      },
                      {
                        "layout": {
                          "root": {
                            "layout": {
                              "leaf": {
                                "key": "0x00efb3a1",
                                "ty": 0
                              }
                            },
                            "root_key": "0x00efb3a1"
                          }
                        },
                        "name": "allowances"
                      }
                    ],
                    "name": "PSP22Data"
                  }
                },
                "name": "data"
              },
              {
                "layout": {
                  "enum": {
                    "dispatchKey": "0x00000000",
                    "name": "Option",
                    "variants": {
                      "0": {
                        "fields": [],
                        "name": "None"
                      },
                      "1": {
                        "fields": [
                          {
                            "layout": {
                              "leaf": {
                                "key": "0x00000000",
                                "ty": 1
                              }
                            },
                            "name": "0"
                          }
                        ],
                        "name": "Some"
                      }
                    }
                  }
                },
                "name": "name"
              },
              {
                "layout": {
                  "enum": {
                    "dispatchKey": "0x00000000",
                    "name": "Option",
                    "variants": {
                      "0": {
                        "fields": [],
                        "name": "None"
                      },
                      "1": {
                        "fields": [
                          {
                            "layout": {
                              "leaf": {
                                "key": "0x00000000",
                                "ty": 1
                              }
                            },
                            "name": "0"
                          }
                        ],
                        "name": "Some"
                      }
                    }
                  }
                },
                "name": "symbol"
              },
              {
                "layout": {
                  "leaf": {
                    "key": "0x00000000",
                    "ty": 2
                  }
                },
                "name": "decimals"
              }
            ],
            "name": "Token"
          }
        },
        "root_key": "0x00000000"
      }
    },
    "types": [
      {
        "id": 0,
        "type": {
          "def": {
            "primitive": "u128"
          }
        }
      },
      {
        "id": 1,
        "type": {
          "def": {
            "primitive": "str"
          }
        }
      },
      {
        "id": 2,
        "type": {
          "def": {
            "primitive": "u8"
          }
        }
      },
      {
        "id": 3,
        "type": {
          "def": {
            "variant": {
              "variants": [
                {
                  "index": 0,
                  "name": "None"
                },
                {
                  "fields": [
                    {
                      "type": 1
                    }
                  ],
                  "index": 1,
                  "name": "Some"
                }
              ]
            }
          },
          "params": [
            {
              "name": "T",
              "type": 1
            }
          ],
          "path": [
            "Option"
          ]
        }
      },
      {
        "id": 4,
        "type": {
          "def": {
            "variant": {
              "variants": [
                {
                  "fields": [
                    {
                      "type": 5
                    }
                  ],
                  "index": 0,
                  "name": "Ok"
                },
                {
                  "fields": [
                    {
                      "type": 6
                    }
                  ],
                  "index": 1,
                  "name": "Err"
                }
              ]
            }
          },
          "params": [
            {
              "name": "T",
              "type": 5
            },
            {
              "name": "E",
              "type": 6
            }
          ],
          "path": [
            "Result"
          ]
        }
      },
      {
        "id": 5,
        "type": {
          "def": {
            "tuple": []
          }
        }
      },
      {
        "id": 6,
        "type": {
          "def": {
            "variant": {
              "variants": [
                {
                  "index": 1,
                  "name": "CouldNotReadInput"
                }
              ]
            }
          },
          "path": [
            "ink_primitives",
            "LangError"
          ]
        }
      },
      {
        "id": 7,
        "type": {
          "def": {
            "variant": {
              "variants": [
                {
                  "fields": [
                    {
                      "type": 0
                    }
                  ],
                  "index": 0,
                  "name": "Ok"
                },
                {
                  "fields": [
                    {
                      "type": 6
                    }
                  ],
                  "index": 1,
                  "name": "Err"
                }
              ]
            }
          },
          "params": [
            {
              "name": "T",
              "type": 0
            },
            {
              "name": "E",
              "type": 6
            }
          ],
          "path": [
            "Result"
          ]
        }
      },
      {
        "id": 8,
        "type": {
          "def": {
            "composite": {
              "fields": [
                {
                  "type": 9,
                  "typeName": "[u8; 32]"
                }
              ]
            }
          },
          "path": [
            "ink_primitives",
            "types",
            "AccountId"
          ]
        }
      },
      {
        "id": 9,
        "type": {
          "def": {
            "array": {
              "len": 32,
              "type": 2
            }
          }
        }
      },
      {
        "id": 10,
        "type": {
          "def": {
            "sequence": {
              "type": 2
            }
          }
        }
      },
      {
        "id": 11,
        "type": {
          "def": {
            "variant": {
              "variants": [
                {
                  "fields": [
                    {
                      "type": 12
                    }
                  ],
                  "index": 0,
                  "name": "Ok"
                },
                {
                  "fields": [
                    {
                      "type": 6
                    }
                  ],
                  "index": 1,
                  "name": "Err"
                }
              ]
            }
          },
          "params": [
            {
              "name": "T",
              "type": 12
            },
            {
              "name": "E",
              "type": 6
            }
          ],
          "path": [
            "Result"
          ]
        }
      },
      {
        "id": 12,
        "type": {
          "def": {
            "variant": {
              "variants": [
                {
                  "fields": [
                    {
                      "type": 5
                    }
                  ],
                  "index": 0,
                  "name": "Ok"
                },
                {
                  "fields": [
                    {
                      "type": 13
                    }
                  ],
                  "index": 1,
                  "name": "Err"
                }
              ]
            }
          },
          "params": [
            {
              "name": "T",
              "type": 5
            },
            {
              "name": "E",
              "type": 13
            }
          ],
          "path": [
            "Result"
          ]
        }
      },
      {
        "id": 13,
        "type": {
          "def": {
            "variant": {
              "variants": [
                {
                  "fields": [
                    {
                      "type": 1,
                      "typeName": "String"
                    }
                  ],
                  "index": 0,
                  "name": "Custom"
                },
                {
                  "index": 1,
                  "name": "InsufficientBalance"
                },
                {
                  "index": 2,
                  "name": "InsufficientAllowance"
                },
                {
                  "index": 3,
                  "name": "ZeroRecipientAddress"
                },
                {
                  "index": 4,
                  "name": "ZeroSenderAddress"
                },
                {
                  "fields": [
                    {
                      "type": 1,
                      "typeName": "String"
                    }
                  ],
                  "index": 5,
                  "name": "SafeTransferCheckFailed"
                }
              ]
            }
          },
          "path": [
            "psp22",
            "errors",
            "PSP22Error"
          ]
        }
      },
      {
        "id": 14,
        "type": {
          "def": {
            "variant": {
              "variants": [
                {
                  "fields": [
                    {
                      "type": 3
                    }
                  ],
                  "index": 0,
                  "name": "Ok"
                },
                {
                  "fields": [
                    {
                      "type": 6
                    }
                  ],
                  "index": 1,
                  "name": "Err"
                }
              ]
            }
          },
          "params": [
            {
              "name": "T",
              "type": 3
            },
            {
              "name": "E",
              "type": 6
            }
          ],
          "path": [
            "Result"
          ]
        }
      },
      {
        "id": 15,
        "type": {
          "def": {
            "variant": {
              "variants": [
                {
                  "fields": [
                    {
                      "type": 2
                    }
                  ],
                  "index": 0,
                  "name": "Ok"
                },
                {
                  "fields": [
                    {
                      "type": 6
                    }
                  ],
                  "index": 1,
                  "name": "Err"
                }
              ]
            }
          },
          "params": [
            {
              "name": "T",
              "type": 2
            },
            {
              "name": "E",
              "type": 6
            }
          ],
          "path": [
            "Result"
          ]
        }
      },
      {
        "id": 16,
        "type": {
          "def": {
            "variant": {
              "variants": [
                {
                  "index": 0,
                  "name": "None"
                },
                {
                  "fields": [
                    {
                      "type": 8
                    }
                  ],
                  "index": 1,
                  "name": "Some"
                }
              ]
            }
          },
          "params": [
            {
              "name": "T",
              "type": 8
            }
          ],
          "path": [
            "Option"
          ]
        }
      },
      {
        "id": 17,
        "type": {
          "def": {
            "composite": {
              "fields": [
                {
                  "type": 9,
                  "typeName": "[u8; 32]"
                }
              ]
            }
          },
          "path": [
            "ink_primitives",
            "types",
            "Hash"
          ]
        }
      },
      {
        "id": 18,
        "type": {
          "def": {
            "primitive": "u64"
          }
        }
      },
      {
        "id": 19,
        "type": {
          "def": {
            "primitive": "u32"
          }
        }
      },
      {
        "id": 20,
        "type": {
          "def": {
            "variant": {}
          },
          "path": [
            "ink_env",
            "types",
            "NoChainExtension"
          ]
        }
      }
    ],
    "version": "4"
  }`
