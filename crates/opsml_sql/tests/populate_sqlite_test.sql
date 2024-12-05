-- Populate opsml_data_registry
INSERT INTO opsml_data_registry (uid, date, timestamp, app_env, name, repository, major, minor, patch, pre_tag, build_tag, contact, tags, data_type, runcard_uid, pipelinecard_uid, auditcard_uid, interface_type) VALUES 
('data1', '2023-11-29', 1701264000, 'development', 'Data1', 'repo1', 1, 0, 0, 'alpha', 'build1', 'contact1', '{"key1": "value1", "key2": "value2"}', 'type1', 'runcard1', 'pipelinecard1', 'auditcard1', 'typeA'),
('data2', '2023-11-29', 1701264001, 'development', 'Data1', 'repo1', 1, 0, 1, 'beta', 'build2', 'contact2', '{}', 'type2', 'runcard2', 'pipelinecard2', 'auditcard2', 'typeB'),
('data3', '2023-11-29', 1701264002, 'development', 'Data1', 'repo1', 1, 1, 0, 'gamma', 'build3', 'contact3', '{}', 'type3', 'runcard3', 'pipelinecard3', 'auditcard3', 'typeC'),
('data4', '2023-11-29', 1701264003, 'development', 'Data1', 'repo1', 1, 1, 1, 'delta', 'build4', 'contact4', '{}', 'type4', 'runcard4', 'pipelinecard4', 'auditcard4', 'typeD'),
('data5', '2023-11-29', 1701264004, 'development', 'Data1', 'repo1', 2, 0, 0, 'epsilon', 'build5', 'contact5', '{}', 'type5', 'runcard5', 'pipelinecard5', 'auditcard5', 'typeE'),
('data6', '2023-11-29', 1701264005, 'development', 'Data1', 'repo1', 2, 0, 1, 'zeta', 'build6', 'contact6', '{}', 'type6', 'runcard6', 'pipelinecard6', 'auditcard6', 'typeF'),
('data7', '2023-11-29', 1701264006, 'development', 'Data1', 'repo1', 2, 1, 0, 'eta', 'build7', 'contact7', '{}', 'type7', 'runcard7', 'pipelinecard7', 'auditcard7', 'typeG'),
('data8', '2023-11-29', 1701264007, 'development', 'Data1', 'repo1', 2, 1, 1, 'theta', 'build8', 'contact8', '{}', 'type8', 'runcard8', 'pipelinecard8', 'auditcard8', 'typeH'),
('data9', '2023-11-29', 1701264008, 'development', 'Data1', 'repo1', 3, 0, 0, 'iota', 'build9', 'contact9', '{}', 'type9', 'runcard9', 'pipelinecard9', 'auditcard9', 'typeI'),
('data10', '2023-11-29', 1701264009, 'development', 'Data1', 'repo1', 3, 0, 1, 'kappa', 'build10', 'contact10', '{}', 'type10', 'runcard10', 'pipelinecard10', 'auditcard10', 'typeJ');

-- Populate opsml_model_registry
INSERT INTO opsml_model_registry (uid, date, timestamp, app_env, name, repository, major, minor, patch, pre_tag, build_tag, contact, tags, datacard_uid, sample_data_type, model_type, runcard_uid, pipelinecard_uid, auditcard_uid, interface_type, task_type) VALUES 
('model1', '2023-11-29', 1701264000, 'development', 'Model1', 'repo1', 1, 0, 0, 'alpha', 'build1', 'contact1', '{}', 'datacard1', 'sample1', 'type1', 'runcard1', 'pipelinecard1', 'auditcard1', 'typeA', 'task1'),
('model2', '2023-11-29', 1701264001, 'development', 'Model2', 'repo2', 1, 0, 1, 'beta', 'build2', 'contact2', '{}', 'datacard2', 'sample2', 'type2', 'runcard2', 'pipelinecard2', 'auditcard2', 'typeB', 'task2'),
('model3', '2023-11-29', 1701264002, 'development', 'Model3', 'repo3', 1, 1, 0, 'gamma', 'build3', 'contact3', '{}', 'datacard3', 'sample3', 'type3', 'runcard3', 'pipelinecard3', 'auditcard3', 'typeC', 'task3'),
('model4', '2023-11-29', 1701264003, 'development', 'Model4', 'repo4', 1, 1, 1, 'delta', 'build4', 'contact4', '{}', 'datacard4', 'sample4', 'type4', 'runcard4', 'pipelinecard4', 'auditcard4', 'typeD', 'task4'),
('model5', '2023-11-29', 1701264004, 'development', 'Model5', 'repo5', 2, 0, 0, 'epsilon', 'build5', 'contact5', '{}', 'datacard5', 'sample5', 'type5', 'runcard5', 'pipelinecard5', 'auditcard5', 'typeE', 'task5'),
('model6', '2023-11-29', 1701264005, 'development', 'Model6', 'repo6', 2, 0, 1, 'zeta', 'build6', 'contact6', '{}', 'datacard6', 'sample6', 'type6', 'runcard6', 'pipelinecard6', 'auditcard6', 'typeF', 'task6'),
('model7', '2023-11-29', 1701264006, 'development', 'Model7', 'repo7', 2, 1, 0, 'eta', 'build7', 'contact7', '{}', 'datacard7', 'sample7', 'type7', 'runcard7', 'pipelinecard7', 'auditcard7', 'typeG', 'task7'),
('model8', '2023-11-29', 1701264007, 'development', 'Model8', 'repo8', 2, 1, 1, 'theta', 'build8', 'contact8', '{}', 'datacard8', 'sample8', 'type8', 'runcard8', 'pipelinecard8', 'auditcard8', 'typeH', 'task8'),
('model9', '2023-11-29', 1701264008, 'development', 'Model9', 'repo9', 3, 0, 0, 'iota', 'build9', 'contact9', '{}', 'datacard9', 'sample9', 'type9', 'runcard9', 'pipelinecard9', 'auditcard9', 'typeI', 'task9'),
('model10', '2023-11-29', 1701264009, 'development', 'Model10', 'repo10', 3, 0, 1, 'kappa', 'build10', 'contact10', '{}', 'datacard10', 'sample10', 'type10', 'runcard10', 'pipelinecard10', 'auditcard10', 'typeJ', 'task10');

-- Populate opsml_run_registry
INSERT INTO opsml_run_registry (uid, date, timestamp, app_env, name, repository, major, minor, patch, pre_tag, build_tag, contact, tags, datacard_uids, modelcard_uids, pipelinecard_uid, project, artifact_uris, compute_environment) VALUES 
('run1', '2023-11-28', 1701264000, 'development', 'Run1', 'repo1', 1, 0, 0, 'alpha', 'build1', 'contact1', '{}', '["datacard1"]', '["modelcard1"]', 'pipelinecard1', 'project1', '{}', '{}'),
('run2', '2023-11-28', 1701264001, 'development', 'Run2', 'repo2', 1, 0, 1, 'beta', 'build2', 'contact2', '{}', '["datacard1"]', '["modelcard1"]', 'pipelinecard2', 'project2', '{}', '{}'),
('run3', '2023-11-29', 1701264002, 'development', 'Run3', 'repo3', 1, 1, 0, 'gamma', 'build3', 'contact3', '{}', '["datacard1"]', '["modelcard1"]', 'pipelinecard3', 'project3', '{}', '{}'),
('run4', '2023-11-29', 1701264003, 'development', 'Run4', 'repo4', 1, 1, 1, 'delta', 'build4', 'contact4', '{}', '["datacard1"]', '["modelcard1"]', 'pipelinecard4', 'project4', '{}', '{}'),
('run5', '2023-11-29', 1701264004, 'development', 'Run5', 'repo5', 2, 0, 0, 'epsilon', 'build5', 'contact5', '{}', '["datacard1"]', '["modelcard1"]', 'pipelinecard5', 'project5', '{}', '{}'),
('run6', '2023-11-29', 1701264005, 'development', 'Run6', 'repo6', 2, 0, 1, 'zeta', 'build6', 'contact6', '{}', '["datacard1"]', '["modelcard1"]', 'pipelinecard6', 'project6', '{}', '{}'),
('run7', '2023-11-29', 1701264006, 'development', 'Run7', 'repo7', 2, 1, 0, 'eta', 'build7', 'contact7', '{}', '["datacard1"]', '["modelcard1"]', 'pipelinecard7', 'project7', '{}', '{}'),
('run8', '2023-11-29', 1701264007, 'development', 'Run8', 'repo8', 2, 1, 1, 'theta', 'build8', 'contact8', '{}', '["datacard1"]', '["modelcard1"]', 'pipelinecard8', 'project8', '{}', '{}'),
('run9', '2023-11-29', 1701264008, 'development', 'Run9', 'repo9', 3, 0, 0, 'iota', 'build9', 'contact9', '{}', '["datacard1"]', '["modelcard1"]', 'pipelinecard9', 'project9', '{}', '{}'),
('run10', '2023-11-29', 1701264009, 'development', 'Run10', 'repo10', 3, 0, 1, 'kappa', 'build10', 'contact10', '{}', '["datacard1"]', '["modelcard1"]', 'pipelinecard10', 'project10', '{}', '{}');

-- Populate opsml_audit_registry
INSERT INTO opsml_audit_registry (uid, date, timestamp, app_env, name, repository, major, minor, patch, pre_tag, build_tag, contact, tags, approved, datacard_uids, modelcard_uids, runcard_uids) VALUES 
('audit1', '2023-11-29', 1701264000, 'development', 'Audit1', 'repo1', 1, 0, 0, 'alpha', 'build1', 'contact1', '{}', 1, '[]', '[]', '[]'), 
('audit2', '2023-11-29', 1701264001, 'development', 'Audit2', 'repo2', 1, 0, 1, 'beta', 'build2', 'contact2', '{}', 0, '[]', '[]', '[]'),
('audit3', '2023-11-29', 1701264002, 'development', 'Audit3', 'repo3', 1, 1, 0, 'gamma', 'build3', 'contact3', '{}', 1, '[]', '[]', '[]'),
('audit4', '2023-11-29', 1701264003, 'development', 'Audit4', 'repo4', 1, 1, 1, 'delta', 'build4', 'contact4', '{}', 0, '[]', '[]', '[]'),
('audit5', '2023-11-29', 1701264004, 'development', 'Audit5', 'repo5', 2, 0, 0, 'epsilon', 'build5', 'contact5', '{}', 1, '[]', '[]', '[]'),
('audit6', '2023-11-29', 1701264005, 'development', 'Audit6', 'repo6', 2, 0, 1, 'zeta', 'build6', 'contact6', '{}', 0, '[]', '[]', '[]'),
('audit7', '2023-11-29', 1701264006, 'development', 'Audit7', 'repo7', 2, 1, 0, 'eta', 'build7', 'contact7', '{}', 1, '[]', '[]', '[]'),
('audit8', '2023-11-29', 1701264007, 'development', 'Audit8', 'repo8', 2, 1, 1, 'theta', 'build8', 'contact8', '{}', 0, '[]', '[]', '[]'),
('audit9', '2023-11-29', 1701264008, 'development', 'Audit9', 'repo9', 3, 0, 0, 'iota', 'build9', 'contact9', '{}', 1, '[]', '[]', '[]'),
('audit10', '2023-11-29', 1701264009, 'development', 'Audit10', 'repo10', 3, 0, 1, 'kappa', 'build10', 'contact10', '{}', 0, '[]', '[]', '[]');